use crate::UiCtx;

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
pub extern "C" fn io_hyprland_go_to_workspace(ui_ctx: &mut UiCtx, idx: usize) {
    ui_ctx
        .rx
        .signal_and_send(Command::HyprlandGoToWorkspace { idx });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_reset(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::LauncherReset);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_up(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::LauncherGoUp);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_down(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::LauncherGoDown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_set_search(ui_ctx: &mut UiCtx, search: *const std::ffi::c_char) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(search) };
    if let Ok(s) = cstr.to_str() {
        ui_ctx.rx.signal_and_send(Command::LauncherSetSearch {
            search: s.to_string(),
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_exec_selected(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::LauncherExecSelected);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::Logout);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(ui_ctx: &mut UiCtx, uuid: *const std::ffi::c_char) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(uuid) };
    if let Ok(s) = cstr.to_str() {
        ui_ctx.rx.signal_and_send(Command::TriggerTray {
            uuid: s.to_string(),
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_network_editor(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::SpawnNetworkEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.signal_and_send(Command::ChangeTheme);
}
