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
mod subscriptions;
mod timer;

use anyhow::Result;
use channel::{CommandsChannel, EventsChannel, VerboseSender};
pub use command::*;
pub use event::Event;
use r#loop::Loop;
use macros::fatal;
pub use subscriptions::*;

pub struct Ctx {
    events: EventsChannel,
    commands: CommandsChannel,
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() -> *mut Ctx {
    env_logger::init();

    let ctx = Ctx {
        commands: CommandsChannel::new(),
        events: EventsChannel::new(),
    };
    Box::leak(Box::new(ctx))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_thread(ctx: *mut Ctx) {
    let mut ctx = unsafe { Box::from_raw(ctx) };

    std::thread::spawn(move || {
        if let Err(err) = io_run_in_place(&mut ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn io_run_in_place(ctx: &mut Ctx) -> Result<()> {
    let tx = ctx.events.tx.clone();
    let rx = ctx.commands.take_rx();

    let r#loop = Loop::new(tx, rx)?;
    r#loop.start();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events(ctx: &mut Ctx, subs: &Subscriptions) {
    while let Some(event) = ctx.events.rx.recv() {
        log::info!("Received event {:?}", event);
        subs.notify_each(&event);
    }
}
