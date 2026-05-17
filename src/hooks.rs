use std::process::Stdio;
use tokio::runtime::Runtime;

/// 钩子事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookEvent {
    Start,
    Break,
    Complete,
}

/// 使用独立的轻量 tokio RT 异步执行钩子脚本。
/// 外部脚本失败不影响主程序——所有错误被静默打印到 stderr。
pub fn fire_hook(cmd: &str, event: HookEvent, task: Option<&str>) {
    let cmd = cmd.to_string();
    let task = task.map(String::from);
    let event_name = match event {
        HookEvent::Start => "start".to_string(),
        HookEvent::Break => "break".to_string(),
        HookEvent::Complete => "complete".to_string(),
    };

    std::thread::spawn(move || {
        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[termato:hook] 创建异步运行时失败: {e}");
                return;
            }
        };
        rt.block_on(async move {
            // Windows 使用 cmd /C，UNIX 使用 sh -c
            let (shell, flag) = if cfg!(windows) {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let mut cmd_builder = tokio::process::Command::new(shell);
            cmd_builder
                .arg(flag)
                .arg(&cmd)
                .env("TERMATO_EVENT", &event_name)
                .env("TERMATO_TASK", task.as_deref().unwrap_or(""))
                .stdout(Stdio::null())
                .stderr(Stdio::null());

            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                cmd_builder.status(),
            )
            .await
            {
                Ok(Ok(status)) if !status.success() => {
                    eprintln!(
                        "[termato:hook] 脚本退出码非零: {} (cmd: {cmd})",
                        status.code().unwrap_or(-1)
                    );
                }
                Ok(Ok(_)) => { /* 成功完成 */ }
                Ok(Err(e)) => {
                    eprintln!("[termato:hook] 脚本执行失败: {e} (cmd: {cmd})");
                }
                Err(_) => {
                    eprintln!("[termato:hook] 脚本超时 (10s)，已放弃: {cmd}");
                }
            }
        });
    });
}
