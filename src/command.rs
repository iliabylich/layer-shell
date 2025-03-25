use crate::Ctx;
use std::ffi::c_void;

#[derive(Debug, Clone)]
#[must_use]
pub enum Command {
    HyprlandGoToWorkspace { idx: usize },

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch { search: String },
    LauncherExecSelected,

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnNetworkEditor,
    SpawnSystemMonitor,
    ChangeTheme,
}

#[unsafe(no_mangle)]
pub extern "C" fn io_hyprland_go_to_workspace(idx: usize, ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::HyprlandGoToWorkspace { idx });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_reset(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherReset);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_up(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherGoUp);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_down(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherGoDown);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_launcher_set_search(search: *const std::ffi::c_char, ctx: *mut c_void) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(search) };
    if let Ok(s) = cstr.to_str() {
        Ctx::from_raw(ctx)
            .commands
            .tx
            .signal_and_send(Command::LauncherSetSearch {
                search: s.to_string(),
            });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_exec_selected(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::LauncherExecSelected);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::Logout);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_trigger_tray(uuid: *const std::ffi::c_char, ctx: *mut c_void) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(uuid) };
    if let Ok(s) = cstr.to_str() {
        Ctx::from_raw(ctx)
            .commands
            .tx
            .signal_and_send(Command::TriggerTray {
                uuid: s.to_string(),
            });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_network_editor(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SpawnNetworkEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme(ctx: *mut c_void) {
    Ctx::from_raw(ctx)
        .commands
        .tx
        .signal_and_send(Command::ChangeTheme);
}
