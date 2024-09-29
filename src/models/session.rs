use crate::{
    models::{Command, Event},
    utils::global,
};
use tokio::sync::mpsc::Sender;

struct Session {
    idx: usize,
    sender: Sender<Event>,
}
global!(SESSION, Session);

pub(crate) async fn spawn(tx: Sender<Event>) {
    let session = Session { idx: 0, sender: tx };
    session.send().await;
    SESSION::set(session);
}

impl Session {
    const MAX: usize = 4;

    async fn reset(&mut self) {
        self.idx = 0;
        self.send().await;
    }
    async fn go_left(&mut self) {
        if self.idx == 0 {
            return;
        }
        self.idx = std::cmp::max(0, self.idx - 1);
        self.send().await;
    }
    async fn go_right(&mut self) {
        self.idx = std::cmp::min(Self::MAX - 1, self.idx + 1);
        self.send().await;
    }
    async fn send(&self) {
        if self
            .sender
            .send(Event::SessionScreen(self.idx))
            .await
            .is_err()
        {
            log::error!("Failed to send SessionScreen event");
        }
    }
}

pub(crate) async fn on_command(command: &Command) {
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
    match command {
        Command::Lock => exec("hyprlock", &[]).await,
        Command::Reboot => exec("systemctl", &["reboot"]).await,
        Command::Shutdown => exec("systemctl", &["poweroff"]).await,
        Command::Logout => exec("hyprctl", &["dispatch", "exit"]).await,

        Command::SessionGoLeft => SESSION::get().go_left().await,
        Command::SessionGoRight => SESSION::get().go_right().await,

        Command::SessionReset => SESSION::get().reset().await,
        _ => {}
    }
}
