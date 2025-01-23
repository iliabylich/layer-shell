use crate::{
    lock_channel::LockChannel,
    modules::{app_list, hyprland, pipewire, tray},
};
use anyhow::{Context as _, Result};
use std::sync::LazyLock;

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

    TriggerTray { uuid: *const u8 },

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

unsafe impl Send for Command {}

static CHANNEL: LazyLock<LockChannel<Command>> = LazyLock::new(LockChannel::new);

impl Command {
    pub(crate) fn send(self) {
        CHANNEL.emit(self);
    }

    pub(crate) fn try_recv() -> Option<Self> {
        CHANNEL.try_recv()
    }

    pub(crate) fn execute(self) {
        log::info!("Running command {:?}", self);
        use Command::*;

        let res = match self {
            HyprlandGoToWorkspace { idx } => hyprland::go_to_workspace(idx),

            AppListGoUp => app_list::go_up(),
            AppListGoDown => app_list::go_down(),
            AppListReset => app_list::reset(),
            AppListExecSelected => app_list::exec_selected(),
            AppListSetSearch { search } => app_list::set_search(search),

            SetVolume { volume } => pipewire::set_volume(volume),
            SetMuted { muted } => pipewire::set_muted(muted),

            Lock => lock(),
            Reboot => reboot(),
            Shutdown => shutdown(),
            Logout => logout(),

            TriggerTray { uuid } => tray::trigger(uuid),

            SpawnNetworkEditor => spawn_network_editor(),
            SpawnSystemMonitor => spawn_system_monitor(),
        };

        if let Err(err) = res {
            log::error!("{:?}", err);
        }
    }
}

fn spawn_network_editor() -> Result<()> {
    spawn("kitty", ["--name", "nmtui", "nmtui"])
}
fn spawn_system_monitor() -> Result<()> {
    spawn("gnome-system-monitor", [])
}
fn lock() -> Result<()> {
    spawn("hyprlock", [])
}
fn reboot() -> Result<()> {
    spawn("systemctl", ["reboot"])
}
fn shutdown() -> Result<()> {
    spawn("systemctl", ["poweroff"])
}
fn logout() -> Result<()> {
    spawn("hyprctl", ["dispatch", "exit"])
}

fn spawn(cmd: &str, args: impl IntoIterator<Item = &'static str>) -> Result<()> {
    std::process::Command::new(cmd)
        .args(args)
        .spawn()
        .with_context(|| format!("failed to spawn {:?}", cmd))?;
    Ok(())
}
