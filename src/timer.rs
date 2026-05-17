use crate::config::Config;
use crate::data::{self, SessionRecord};
use crate::hooks::HookEvent;
use chrono::{DateTime, Local};
use std::time::{Duration, Instant};

/// 计时器所处的阶段
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

/// 全局运行状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Idle,
    Running,
    Paused,
}

/// tick() 的返回值：(阶段是否完成, 需要触发的钩子事件)
pub type TickResult = (bool, Option<HookEvent>);

/// 番茄钟状态机
pub struct PomodoroTimer {
    pub phase: Phase,
    pub state: State,
    total_duration: Duration,
    phase_start: Option<Instant>,
    paused_elapsed: Duration,
    completed_focus_count: u32,
    pub task_name: Option<String>,
    session_started_at: Option<DateTime<Local>>,
    config: Config,
    /// 任务队列：专注阶段开始时自动弹出下一个
    task_queue: Vec<String>,
}

impl PomodoroTimer {
    pub fn new(config: Config, task_name: Option<String>, queue: Vec<String>) -> Self {
        // 如果没手动指定任务名，尝试从队列弹出
        let task_name = task_name.or_else(|| {
            if queue.is_empty() {
                None
            } else {
                Some(queue[0].clone())
            }
        });
        // 第一个任务作为当前任务，剩余保留在队列
        let task_queue = if task_name.is_some() && !queue.is_empty() {
            queue[1..].to_vec()
        } else {
            queue
        };

        Self {
            phase: Phase::Focus,
            state: State::Idle,
            total_duration: Duration::from_secs(config.timer.focus_minutes * 60),
            phase_start: None,
            paused_elapsed: Duration::ZERO,
            completed_focus_count: 0,
            task_name,
            session_started_at: None,
            config,
            task_queue,
        }
    }

    /// 开始或恢复计时
    pub fn start(&mut self) {
        if self.state == State::Idle {
            self.session_started_at = Some(Local::now());
        }
        self.state = State::Running;
        self.phase_start = Some(Instant::now());
    }

    /// 暂停计时
    pub fn pause(&mut self) {
        if self.state == State::Running {
            self.paused_elapsed += self.phase_start
                .map(|s| s.elapsed())
                .unwrap_or(Duration::ZERO);
            self.state = State::Paused;
            self.phase_start = None;
        }
    }

    /// 切换暂停/继续
    pub fn toggle_pause(&mut self) {
        match self.state {
            State::Running => self.pause(),
            State::Paused => self.start(),
            _ => {}
        }
    }

    /// 重置当前阶段
    pub fn reset(&mut self) {
        self.state = State::Idle;
        self.phase_start = None;
        self.paused_elapsed = Duration::ZERO;
        self.session_started_at = None;
    }

    /// 跳过当前阶段
    pub fn skip(&mut self) -> TickResult {
        self.record_session("skipped");
        let event = self.advance_phase();
        (true, Some(event))
    }

    pub fn elapsed(&self) -> Duration {
        let from_pause = self.paused_elapsed;
        let from_run = self
            .phase_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        from_pause + from_run
    }

    pub fn remaining(&self) -> Duration {
        self.total_duration.saturating_sub(self.elapsed())
    }

    pub fn progress(&self) -> f64 {
        if self.total_duration.as_secs() == 0 {
            return 1.0;
        }
        (self.elapsed().as_secs_f64() / self.total_duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    /// 检查当前阶段是否完成
    pub fn tick(&mut self) -> TickResult {
        if self.state != State::Running {
            return (false, None);
        }
        if self.remaining() == Duration::ZERO {
            self.record_session("completed");
            let event = self.advance_phase();
            return (true, Some(event));
        }
        (false, None)
    }

    /// 推进到下一个阶段，返回触发的钩子事件
    fn advance_phase(&mut self) -> HookEvent {
        let event = match self.phase {
            Phase::Focus => {
                self.completed_focus_count += 1;
                if self.completed_focus_count % self.config.timer.rounds_before_long_break == 0 {
                    self.phase = Phase::LongBreak;
                    self.total_duration =
                        Duration::from_secs(self.config.timer.long_break_minutes * 60);
                } else {
                    self.phase = Phase::ShortBreak;
                    self.total_duration =
                        Duration::from_secs(self.config.timer.short_break_minutes * 60);
                }
                HookEvent::Complete
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Focus;
                self.total_duration = Duration::from_secs(self.config.timer.focus_minutes * 60);
                // 自动弹出下一个任务
                self.pop_next_task();
                HookEvent::Break
            }
        };

        self.state = State::Idle;
        self.phase_start = None;
        self.paused_elapsed = Duration::ZERO;
        self.session_started_at = None;
        event
    }

    /// 从队列弹出下一个任务
    fn pop_next_task(&mut self) {
        if !self.task_queue.is_empty() {
            self.task_name = Some(self.task_queue.remove(0));
        }
    }

    fn record_session(&mut self, status: &str) {
        if let Some(started_at) = self.session_started_at.take() {
            let record = SessionRecord {
                started_at,
                duration_secs: self.elapsed().as_secs(),
                task: self.task_name.clone(),
                status: status.to_string(),
                kind: match self.phase {
                    Phase::Focus => "focus".to_string(),
                    Phase::ShortBreak => "short_break".to_string(),
                    Phase::LongBreak => "long_break".to_string(),
                },
            };
            if let Err(e) = data::append_record(&record) {
                eprintln!("[termato] 写入日志失败: {e}");
            }
        }
    }

    pub fn phase_label(&self) -> &str {
        match self.phase {
            Phase::Focus => "Focus",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }

    pub fn completed_count(&self) -> u32 {
        self.completed_focus_count
    }

    pub fn pending_count(&self) -> usize {
        self.task_queue.len()
    }

    pub fn record_session_on_quit(&mut self) {
        self.record_session("interrupted");
    }
}
