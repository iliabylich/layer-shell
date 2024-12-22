use crate::actors::session;
use layer_shell_app_list::{
    AppListExecSelected, AppListGoDown, AppListGoUp, AppListReset, AppListSetSearch,
};
use layer_shell_hyprland::HyprlandGoToWorkspace;
use layer_shell_pipewire::{SetMuted, SetVolume};
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum Command {
    HyprlandGoToWorkspace(HyprlandGoToWorkspace),

    AppListReset(AppListReset),
    AppListGoUp(AppListGoUp),
    AppListGoDown(AppListGoDown),
    AppListSetSearch(AppListSetSearch),
    AppListExecSelected(AppListExecSelected),

    SetVolume(SetVolume),
    SetMuted(SetMuted),

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
    async fn execute(self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        match self {
            SetVolume(cmd) => cmd.exec().await,
            SetMuted(cmd) => cmd.exec().await,

            HyprlandGoToWorkspace(cmd) => cmd.exec().await,

            AppListGoUp(cmd) => cmd.exec().await,
            AppListGoDown(cmd) => cmd.exec().await,
            AppListReset(cmd) => cmd.exec().await,
            AppListExecSelected(cmd) => cmd.exec().await,
            AppListSetSearch(cmd) => cmd.exec().await,

            Lock | Reboot | Shutdown | Logout => session::on_command(self).await,

            SpawnNetworkEditor => spawn_network_editor(),
            SpawnSystemMonitor => spawn_system_monitor(),
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
