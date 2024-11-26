use crate::actors::{app_list, hyprland, pipewire, session};
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum Command {
    GoToWorkspace(usize),

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch(String),
    LauncherExecSelected,

    SetVolume(f32),
    SetMuted(bool),

    Lock,
    Reboot,
    Shutdown,
    Logout,

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

pub(crate) async fn start_processing(rx: Receiver<Command>) {
    loop {
        while let Ok(command) = rx.try_recv() {
            command.execute().await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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

            Lock | Reboot | Shutdown | Logout => session::on_command(self).await,

            SpawnNetworkEditor => spawn_network_editor(),
            SpawnSystemMonitor => spawn_system_monitor(),

            SetVolume(_) | SetMuted(_) => pipewire::on_command(self).await,
        }
    }
}

fn spawn_network_editor() {
    if let Err(err) = std::process::Command::new("kitty")
        .args(["--name", "nmtui", "nmtui"])
        .spawn()
    {
        log::error!("failed to spawn kitty: {:?}", err);
    }
}

fn spawn_system_monitor() {
    if let Err(err) = std::process::Command::new("gnome-system-monitor").spawn() {
        log::error!("failed to spawn gnome-system-monitor: {:?}", err);
    }
}

impl TryFrom<&Command> for layer_shell_pipewire::Command {
    type Error = ();

    fn try_from(cmd: &Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::SetVolume(volume) => Ok(Self::SetVolume(*volume)),
            Command::SetMuted(muted) => Ok(Self::SetMuted(*muted)),
            _ => Err(()),
        }
    }
}
