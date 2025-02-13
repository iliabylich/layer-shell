use crate::{macros::fatal, Ctx};
use std::ffi::c_void;

#[derive(Debug, Clone)]
pub enum Command {
    HyprlandGoToWorkspace { idx: usize },

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch { search: String },
    LauncherExecSelected,

    SetVolume { volume: f64 },
    SetMuted { muted: bool },

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnNetworkEditor,
    SpawnSystemMonitor,
}

#[no_mangle]
pub extern "C" fn layer_shell_io_hyprland_go_to_workspace(idx: usize, ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::HyprlandGoToWorkspace { idx });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_launcher_reset(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherReset);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_launcher_go_up(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherGoUp);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_launcher_go_down(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherGoDown);
}
#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_launcher_set_search(
    search: *const std::ffi::c_char,
    ctx: *mut c_void,
) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(search) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 Launcher search"));
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherSetSearch {
            search: s.to_string(),
        });
}
#[no_mangle]
pub extern "C" fn layer_shell_io_launcher_exec_selected(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherExecSelected);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_set_volume(volume: f64, ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SetVolume { volume });
}
#[no_mangle]
pub extern "C" fn layer_shell_io_set_muted(muted: bool, ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SetMuted { muted });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_lock(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Lock);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_reboot(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Reboot);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_shutdown(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Shutdown);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_logout(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Logout);
}

#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_trigger_tray(
    uuid: *const std::ffi::c_char,
    ctx: *mut c_void,
) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(uuid) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 Launcher search"));
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::TriggerTray {
            uuid: s.to_string(),
        });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_network_editor(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SpawnNetworkEditor);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_system_monitor(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SpawnSystemMonitor);
}
