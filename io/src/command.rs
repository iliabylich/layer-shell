use layer_shell_app_list::{
    AppListExecSelected, AppListGoDown, AppListGoUp, AppListReset, AppListSetSearch,
};
use layer_shell_hyprland::HyprlandGoToWorkspace;
use layer_shell_pipewire::SetVolume;
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

            HyprlandGoToWorkspace(cmd) => cmd.exec().await,

            AppListGoUp(cmd) => cmd.exec().await,
            AppListGoDown(cmd) => cmd.exec().await,
            AppListReset(cmd) => cmd.exec().await,
            AppListExecSelected(cmd) => cmd.exec().await,
            AppListSetSearch(cmd) => cmd.exec().await,

            Lock => lock(),
            Reboot => reboot(),
            Shutdown => shutdown(),
            Logout => logout(),

            SpawnNetworkEditor => spawn_network_editor(),
            SpawnSystemMonitor => spawn_system_monitor(),
        }
    }
}

fn spawn_network_editor() {
    spawn("kitty", ["--name", "nmtui", "nmtui"]);
}
fn spawn_system_monitor() {
    spawn("gnome-system-monitor", []);
}
fn lock() {
    spawn("hyprlock", []);
}
fn reboot() {
    spawn("systemctl", ["reboot"]);
}
fn shutdown() {
    spawn("systemctl", ["poweroff"]);
}
fn logout() {
    spawn("hyprctl", ["dispatch", "exit"]);
}

fn spawn(cmd: &str, args: impl IntoIterator<Item = &'static str>) {
    if let Err(err) = std::process::Command::new(cmd).args(args).spawn() {
        log::error!("failed to spawn {:?}: {:?}", cmd, err);
    }
}
