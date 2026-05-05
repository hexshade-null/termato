use crate::config::Config;
use crate::data::{self, SessionRecord};
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

/// 番茄钟状态机，驱动整个计时逻辑
pub struct PomodoroTimer {
    pub phase: Phase,
    pub state: State,
    /// 当前阶段的总时长
    total_duration: Duration,
    /// 阶段开始时刻
    phase_start: Option<Instant>,
    /// 已暂停累积的时间
    paused_elapsed: Duration,
    /// 已完成的专注次数（用于判断是否触发长休息）
    completed_focus_count: u32,
    /// 用户指定的任务名
    pub task_name: Option<String>,
    /// 当前阶段开始的时间戳（用于写入日志）
    session_started_at: Option<DateTime<Local>>,
    /// 用户配置
    config: Config,
}

impl PomodoroTimer {
    pub fn new(config: Config, task_name: Option<String>) -> Self {
        let timer = Self {
            phase: Phase::Focus,
            state: State::Idle,
            total_duration: Duration::from_secs(config.timer.focus_minutes * 60),
            phase_start: None,
            paused_elapsed: Duration::ZERO,
            completed_focus_count: 0,
            task_name,
            session_started_at: None,
            config,
        };
        timer
    }

    /// 开始或恢复计时
    pub fn start(&mut self) {
        if self.state == State::Idle {
            self.session_started_at = Some(Local::now());
        }
        self.state = State::Running;
        self.phase_start = Some(Instant::now());
    }

    /// 暂停计时，保存已流逝时间
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

    /// 重置当前阶段回到初始状态
    pub fn reset(&mut self) {
        self.state = State::Idle;
        self.phase_start = None;
        self.paused_elapsed = Duration::ZERO;
        self.session_started_at = None;
    }

    /// 跳过当前阶段，记录后进入下一阶段
    pub fn skip(&mut self) {
        self.record_session("skipped");
        self.advance_phase();
    }

    /// 返回当前阶段已流逝的时间
    pub fn elapsed(&self) -> Duration {
        let from_pause = self.paused_elapsed;
        let from_run = self
            .phase_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        from_pause + from_run
    }

    /// 返回剩余时间
    pub fn remaining(&self) -> Duration {
        self.total_duration.saturating_sub(self.elapsed())
    }

    /// 返回进度比例 [0.0, 1.0]
    pub fn progress(&self) -> f64 {
        if self.total_duration.as_secs() == 0 {
            return 1.0;
        }
        let elapsed = self.elapsed().as_secs_f64();
        let total = self.total_duration.as_secs_f64();
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// 检查当前阶段是否已完成；若完成则记录并切换到下一阶段
    /// 返回 true 表示刚完成了一个阶段
    pub fn tick(&mut self) -> bool {
        if self.state != State::Running {
            return false;
        }
        if self.remaining() == Duration::ZERO {
            self.record_session("completed");
            self.advance_phase();
            return true;
        }
        false
    }

    /// 推进到下一个阶段（focus -> break -> focus ...）
    fn advance_phase(&mut self) {
        match self.phase {
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
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Focus;
                self.total_duration = Duration::from_secs(self.config.timer.focus_minutes * 60);
            }
        }
        // 重置阶段计时
        self.state = State::Idle;
        self.phase_start = None;
        self.paused_elapsed = Duration::ZERO;
        self.session_started_at = None;
    }

    /// 将完成的会话写入日志
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
            // 静默写入，日志失败不应中断计时
            if let Err(e) = data::append_record(&record) {
                eprintln!("[termato] 写入日志失败: {e}");
            }
        }
    }

    /// 当前阶段的可读标签
    pub fn phase_label(&self) -> &str {
        match self.phase {
            Phase::Focus => "Focus",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }

    /// 当前完成的番茄总数
    pub fn completed_count(&self) -> u32 {
        self.completed_focus_count
    }

    /// 退出时将当前进行中的会话标记为 interrupted 并写入日志
    pub fn record_session_on_quit(&mut self) {
        self.record_session("interrupted");
    }
}
