use crate::modules::{
    app_list::command::{exec_selected, go_down, go_up, reset, set_search},
    hyprland::command::go_to_workspace,
    pipewire::command::set_volume,
};
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
#[repr(C)]
pub enum Command {
    HyprlandGoToWorkspace { idx: usize },
    AppListReset,
    AppListGoUp,
    AppListGoDown,
    AppListSetSearch { search: *const u8 },
    AppListExecSelected,

    SetVolume { volume: f64 },
    Lock,
    Reboot,
    Shutdown,
    Logout,

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

unsafe impl Send for Command {}

pub(crate) async fn start_processing(mut rx: Receiver<Command>) {
    while let Some(command) = rx.recv().await {
        command.execute().await;
    }
}

impl Command {
    async fn execute(self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        match self {
            SetVolume { volume } => set_volume(volume),
            HyprlandGoToWorkspace { idx } => go_to_workspace(idx).await,
            AppListGoUp => go_up().await,
            AppListGoDown => go_down().await,
            AppListReset => reset().await,
            AppListExecSelected => exec_selected().await,
            AppListSetSearch { search } => set_search(search).await,

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
