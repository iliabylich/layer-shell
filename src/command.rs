use crate::lock_channel::LockChannel;
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
}

pub(crate) fn spawn_network_editor() -> Result<()> {
    spawn("kitty", ["--name", "nmtui", "nmtui"])
}
pub(crate) fn spawn_system_monitor() -> Result<()> {
    spawn("gnome-system-monitor", [])
}
pub(crate) fn lock() -> Result<()> {
    spawn("hyprlock", [])
}
pub(crate) fn reboot() -> Result<()> {
    spawn("systemctl", ["reboot"])
}
pub(crate) fn shutdown() -> Result<()> {
    spawn("systemctl", ["poweroff"])
}
pub(crate) fn logout() -> Result<()> {
    spawn("hyprctl", ["dispatch", "exit"])
}

fn spawn(cmd: &str, args: impl IntoIterator<Item = &'static str>) -> Result<()> {
    std::process::Command::new(cmd)
        .args(args)
        .spawn()
        .with_context(|| format!("failed to spawn {:?}", cmd))?;
    Ok(())
}
