#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]
#![expect(clippy::not_unsafe_ptr_arg_deref)]

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
mod subscriptions;
mod timer;

use anyhow::Result;
use channel::{CommandReceiver, CommandSender, EventReceiver, EventSender};
pub use command::*;
pub use event::Event;
use r#loop::Loop;
use macros::fatal;
pub use subscriptions::*;

#[repr(C)]
pub struct Ctx {
    pub io: *mut IoCtx,
    pub ui: *mut UiCtx,
}

pub struct IoCtx {
    tx: EventSender,
    rx: CommandReceiver,
}
pub struct UiCtx {
    tx: EventReceiver,
    rx: CommandSender,
    subs: Subscriptions,
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() -> Ctx {
    env_logger::init();

    let (etx, erx) = channel::events();
    let (ctx, crx) = channel::commands();
    let subs = Subscriptions::new();

    Ctx {
        io: Box::leak(Box::new(IoCtx { tx: etx, rx: crx })),
        ui: Box::leak(Box::new(UiCtx {
            tx: erx,
            rx: ctx,
            subs,
        })),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_subscribe(
    ui_ctx: &mut UiCtx,
    f: extern "C" fn(&Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
) {
    ui_ctx.subs.push(f, data);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_thread(io_ctx: *mut IoCtx) {
    let io_ctx = unsafe { *Box::from_raw(io_ctx) };

    std::thread::spawn(move || {
        if let Err(err) = io_run_in_place(io_ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn io_run_in_place(io_ctx: IoCtx) -> Result<()> {
    let tx = io_ctx.tx;
    let rx = io_ctx.rx;

    let r#loop = Loop::new(tx, rx)?;
    r#loop.start();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events(ui_ctx: &mut UiCtx) {
    while let Some(event) = ui_ctx.tx.recv() {
        log::info!("Received event {:?}", event);
        ui_ctx.subs.notify_each(&event);
    }
}
