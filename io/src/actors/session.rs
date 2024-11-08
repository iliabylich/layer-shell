use crate::Command;

pub(crate) async fn on_command(command: &Command) {
    match command {
        Command::Lock => exec("hyprlock", &[]).await,
        Command::Reboot => exec("systemctl", &["reboot"]).await,
        Command::Shutdown => exec("systemctl", &["poweroff"]).await,
        Command::Logout => exec("hyprctl", &["dispatch", "exit"]).await,

        _ => {}
    }
}

async fn exec(cmd: &str, args: &[&str]) {
    match tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Err(err) = child.wait().await {
                log::error!("spawned {cmd} has failed: {}", err);
            }
        }

        Err(err) => {
            log::error!("failed to spawn {cmd}: {}", err)
        }
    }
}
