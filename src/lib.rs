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
mod timer;

use anyhow::Result;
use channel::{CommandsChannel, EventsChannel, VerboseSender};
pub use command::*;
pub use event::Event;
use r#loop::Loop;
use macros::fatal;

type Subscriptions = Vec<(
    extern "C" fn(*const Event, *mut std::ffi::c_void),
    *mut std::ffi::c_void,
)>;

pub struct Ctx {
    subscriptions: Subscriptions,
    events: EventsChannel,
    commands: CommandsChannel,
}
impl Ctx {
    pub(crate) fn from_raw(ctx: *mut Self) -> &'static mut Self {
        let ctx = unsafe { ctx.as_mut() };
        ctx.unwrap_or_else(|| fatal!("Can't read NULL ctx"))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() -> *mut Ctx {
    env_logger::init();

    let ctx = Ctx {
        subscriptions: vec![],
        commands: CommandsChannel::new(),
        events: EventsChannel::new(),
    };
    Box::leak(Box::new(ctx))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_subscribe(
    f: extern "C" fn(*const Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
    ctx: *mut Ctx,
) {
    Ctx::from_raw(ctx).subscriptions.push((f, data));
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_thread(ctx: *mut Ctx) {
    struct SendPtr {
        ptr: *mut Ctx,
    }
    unsafe impl Send for SendPtr {}
    impl SendPtr {
        fn get(&self) -> *mut Ctx {
            self.ptr
        }
    }
    let ctx = SendPtr { ptr: ctx };

    std::thread::spawn(move || {
        let ctx: *mut Ctx = ctx.get();

        if let Err(err) = io_run_in_place(ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn io_run_in_place(ctx: *mut Ctx) -> Result<()> {
    let ctx = Ctx::from_raw(ctx);
    let tx = ctx.events.tx.clone();
    let rx = ctx.commands.take_rx();

    let r#loop = Loop::new(tx, rx)?;
    r#loop.start();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events(ctx: *mut Ctx) {
    let ctx = Ctx::from_raw(ctx);
    while let Some(event) = ctx.events.rx.recv() {
        log::info!("Received event {:?}", event);

        for (sub, data) in ctx.subscriptions.iter() {
            sub(&event, *data);
        }
    }
}
