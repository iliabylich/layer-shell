use crate::{
    fatal::fatal,
    modules::{app_list, hyprland, pipewire},
};
use std::sync::{mpsc::Sender, OnceLock};

#[derive(Debug)]
#[repr(C)]
pub enum Command {
    HyprlandGoToWorkspace { idx: usize },
    AppListReset,
    AppListGoUp,
    AppListGoDown,
    AppListSetSearch { search: *const u8 },
    AppListExecSelected,

    SetVolume { volume: f32 },
    SetMuted { muted: bool },
    Lock,
    Reboot,
    Shutdown,
    Logout,

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

static SENDER: OnceLock<Sender<Command>> = OnceLock::new();

unsafe impl Send for Command {}

impl Command {
    pub(crate) fn set_sender(sender: Sender<Command>) {
        if SENDER.set(sender).is_err() {
            fatal!("Command sender has already been set");
        }
    }

    pub(crate) fn send(self) {
        match SENDER.get() {
            Some(sender) => {
                if let Err(err) = sender.send(self) {
                    log::error!("failed to publish event: {:?}", err);
                }
            }
            None => {
                fatal!("Command sender is not set");
            }
        }
    }

    pub(crate) fn execute(self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        match self {
            SetVolume { volume } => pipewire::set_volume(volume),
            SetMuted { muted } => pipewire::set_muted(muted),
            HyprlandGoToWorkspace { idx } => hyprland::go_to_workspace(idx),
            AppListGoUp => app_list::go_up(),
            AppListGoDown => app_list::go_down(),
            AppListReset => app_list::reset(),
            AppListExecSelected => app_list::exec_selected(),
            AppListSetSearch { search } => app_list::set_search(search),
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
