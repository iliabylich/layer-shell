use crate::{app_list, hyprland, output_sound, session};
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum Command {
    GoToWorkspace(usize),

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch(String),
    LauncherExecSelected,

    SetVolume(f64),

    Lock,
    Reboot,
    Shutdown,
    Logout,
    SessionGoLeft,
    SessionGoRight,
    SessionReset,

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

pub(crate) async fn start_processing(mut rx: Receiver<Command>) {
    while let Some(command) = rx.recv().await {
        command.execute().await;
    }
}

impl Command {
    async fn execute(&self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        match self {
            GoToWorkspace(_) => hyprland::on_command(self).await,

            LauncherReset | LauncherGoUp | LauncherGoDown | LauncherSetSearch(_)
            | LauncherExecSelected => app_list::on_command(self).await,

            SetVolume(_) => output_sound::on_command(self).await,

            Lock | Reboot | Shutdown | Logout | SessionGoLeft | SessionGoRight | SessionReset => {
                session::on_command(self).await
            }

            SpawnNetworkEditor => spawn_network_editor(),
            SpawnSystemMonitor => spawn_system_monitor(),
        }
    }
}

fn spawn_network_editor() {
    std::process::Command::new("kitty")
        .args(&["--name", "nmtui", "nmtui"])
        .spawn()
        .unwrap();
}

fn spawn_system_monitor() {
    std::process::Command::new("gnome-system-monitor")
        .spawn()
        .unwrap();
}
