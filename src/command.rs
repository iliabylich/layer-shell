use crate::{ffi::CString, lock_channel::LockChannel};
use std::sync::LazyLock;

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Command {
    HyprlandGoToWorkspace { idx: usize },

    AppListReset,
    AppListGoUp,
    AppListGoDown,
    AppListSetSearch { search: CString },
    AppListExecSelected,

    SetVolume { volume: f32 },
    SetMuted { muted: bool },

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: CString },

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
