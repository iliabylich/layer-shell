use crate::{lock_channel::LockChannel, macros::fatal};
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub(crate) enum Command {
    HyprlandGoToWorkspace { idx: usize },

    AppListReset,
    AppListGoUp,
    AppListGoDown,
    AppListSetSearch { search: String },
    AppListExecSelected,

    SetVolume { volume: f32 },
    SetMuted { muted: bool },

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnNetworkEditor,
    SpawnSystemMonitor,

    Probe,
}

static CHANNEL: LazyLock<LockChannel<Command>> = LazyLock::new(LockChannel::new);

impl Command {
    pub(crate) fn try_recv() -> Option<Self> {
        CHANNEL.try_recv()
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_hyprland_go_to_workspace(idx: usize) {
    CHANNEL.emit(Command::HyprlandGoToWorkspace { idx })
}

#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_reset() {
    CHANNEL.emit(Command::AppListReset);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_go_up() {
    CHANNEL.emit(Command::AppListGoUp);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_go_down() {
    CHANNEL.emit(Command::AppListGoDown);
}
#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_app_list_set_search(search: *const std::ffi::c_char) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(search) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 AppList search"));
    CHANNEL.emit(Command::AppListSetSearch {
        search: s.to_string(),
    });
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_exec_selected() {
    CHANNEL.emit(Command::AppListExecSelected);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_set_volume(volume: f32) {
    CHANNEL.emit(Command::SetVolume { volume });
}
#[no_mangle]
pub extern "C" fn layer_shell_io_set_muted(muted: bool) {
    CHANNEL.emit(Command::SetMuted { muted });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_lock() {
    CHANNEL.emit(Command::Lock);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_reboot() {
    CHANNEL.emit(Command::Reboot);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_shutdown() {
    CHANNEL.emit(Command::Shutdown);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_logout() {
    CHANNEL.emit(Command::Logout);
}

#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_trigger_tray(uuid: *const std::ffi::c_char) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(uuid) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 AppList search"));
    CHANNEL.emit(Command::TriggerTray {
        uuid: s.to_string(),
    });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_network_editor() {
    CHANNEL.emit(Command::SpawnNetworkEditor);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_system_monitor() {
    CHANNEL.emit(Command::SpawnSystemMonitor);
}
