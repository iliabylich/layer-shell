#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod channel;
mod command;
mod dbus;
mod event;
mod fd_id;
mod ffi;
mod hyprctl;
mod r#loop;
mod macros;
mod modules;
mod poll;
mod timer;

use anyhow::Result;
use channel::{CommandReceiver, CommandSender, EventReceiver, EventSender};
use command::*;
use event::Event;
use ffi::{CArray, CString};
use r#loop::Loop;
use macros::fatal;

pub struct IoCtx {
    tx: EventSender,
    rx: CommandReceiver,
}
pub struct UiCtx {
    tx: EventReceiver,
    rx: CommandSender,
}
impl Drop for UiCtx {
    fn drop(&mut self) {
        eprintln!("Dropping UiCtx...");
    }
}
#[repr(C)]
pub struct Ctx {
    pub io: *mut IoCtx,
    pub ui: *mut UiCtx,
}

pub struct IoThread {
    handle: std::thread::JoinHandle<()>,
}

/// # Safety
///
/// This function must be called at most once.
pub unsafe fn io_run_in_place(io_ctx: *mut IoCtx) -> Result<()> {
    let io_ctx = unsafe { *Box::from_raw(io_ctx) };

    let tx = io_ctx.tx;
    let rx = io_ctx.rx;

    let r#loop = Loop::new(tx, rx)?;
    r#loop.start();
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() -> Ctx {
    env_logger::init();

    let (etx, erx) = channel::events();
    let (ctx, crx) = channel::commands();

    let io_ctx = IoCtx { tx: etx, rx: crx };
    let ui_ctx = UiCtx { tx: erx, rx: ctx };

    Ctx {
        io: Box::leak(Box::new(io_ctx)),
        ui: Box::leak(Box::new(ui_ctx)),
    }
}
/// # Safety
///
/// This function must be called at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_spawn_thread(io_ctx: *mut IoCtx) -> *mut IoThread {
    let io_ctx = unsafe { Box::from_raw(io_ctx) };

    let handle = std::thread::spawn(move || {
        if let Err(err) = unsafe { io_run_in_place(Box::leak(io_ctx)) } {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });

    Box::leak(Box::new(IoThread { handle }))
}
#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events(ui_ctx: &mut UiCtx) -> CArray<Event> {
    let mut out = vec![];
    while let Some(event) = ui_ctx.tx.recv() {
        log::info!("Received event {:?}", event);
        out.push(event);
    }
    out.into()
}
#[unsafe(no_mangle)]
pub extern "C" fn io_drop_events(events: CArray<Event>) {
    drop(events)
}
#[unsafe(no_mangle)]
pub extern "C" fn io_finalize(ui_ctx: *mut UiCtx, io_thread: *mut IoThread) {
    let mut ui_ctx = unsafe { Box::from_raw(ui_ctx) };
    ui_ctx.rx.send(Command::FinishIoThread);

    let io_thread = unsafe { Box::from_raw(io_thread) };
    eprintln!("Waiting for IO thread to finish...");
    io_thread.handle.join().unwrap();
    eprintln!("IO thread has finished...");

    drop(ui_ctx);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_hyprland_go_to_workspace(ui_ctx: &mut UiCtx, idx: usize) {
    ui_ctx.rx.send(Command::HyprlandGoToWorkspace { idx });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_reset(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::LauncherReset);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_up(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::LauncherGoUp);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_go_down(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::LauncherGoDown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_set_search(ui_ctx: &mut UiCtx, search: *const u8) {
    ui_ctx.rx.send(Command::LauncherSetSearch {
        search: CString::from(search).into(),
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_launcher_exec_selected(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::LauncherExecSelected);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_lock(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::Logout);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(ui_ctx: &mut UiCtx, uuid: *const u8) {
    ui_ctx.rx.send(Command::TriggerTray {
        uuid: CString::from(uuid).into(),
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_network_editor(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::SpawnNetworkEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme(ui_ctx: &mut UiCtx) {
    ui_ctx.rx.send(Command::ChangeTheme);
}
