use crate::macros::fatal;
use std::ffi::c_void;

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

macro_rules! send {
    ($ctx:ident, $cmd:expr) => {
        if let Err(err) = $crate::cast_ctx!($ctx).cmd_tx.send($cmd) {
            log::error!("Failed to send command: {:?}", err)
        }
    };
}

#[no_mangle]
pub extern "C" fn layer_shell_io_hyprland_go_to_workspace(idx: usize, ctx: *mut c_void) {
    send!(ctx, Command::HyprlandGoToWorkspace { idx });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_reset(ctx: *mut c_void) {
    send!(ctx, Command::AppListReset);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_go_up(ctx: *mut c_void) {
    send!(ctx, Command::AppListGoUp);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_go_down(ctx: *mut c_void) {
    send!(ctx, Command::AppListGoDown);
}
#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_app_list_set_search(
    search: *const std::ffi::c_char,
    ctx: *mut c_void,
) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(search) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 AppList search"));
    send!(
        ctx,
        Command::AppListSetSearch {
            search: s.to_string(),
        }
    );
}
#[no_mangle]
pub extern "C" fn layer_shell_io_app_list_exec_selected(ctx: *mut c_void) {
    send!(ctx, Command::AppListExecSelected);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_set_volume(volume: f32, ctx: *mut c_void) {
    send!(ctx, Command::SetVolume { volume });
}
#[no_mangle]
pub extern "C" fn layer_shell_io_set_muted(muted: bool, ctx: *mut c_void) {
    send!(ctx, Command::SetMuted { muted });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_lock(ctx: *mut c_void) {
    send!(ctx, Command::Lock);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_reboot(ctx: *mut c_void) {
    send!(ctx, Command::Reboot);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_shutdown(ctx: *mut c_void) {
    send!(ctx, Command::Shutdown);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_logout(ctx: *mut c_void) {
    send!(ctx, Command::Logout);
}

#[no_mangle]
pub unsafe extern "C" fn layer_shell_io_trigger_tray(
    uuid: *const std::ffi::c_char,
    ctx: *mut c_void,
) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(uuid) };
    let s = cstr
        .to_str()
        .unwrap_or_else(|_| fatal!("non-utf8 AppList search"));
    send!(
        ctx,
        Command::TriggerTray {
            uuid: s.to_string(),
        }
    );
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_network_editor(ctx: *mut c_void) {
    send!(ctx, Command::SpawnNetworkEditor);
}
#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_system_monitor(ctx: *mut c_void) {
    send!(ctx, Command::SpawnSystemMonitor);
}
