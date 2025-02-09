#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]
#![expect(clippy::missing_safety_doc)]

mod command;
mod dbus;
mod event;
mod ffi;
mod hyprctl;
mod lock_channel;
mod logger;
mod macros;
mod modules;
mod scheduler;

pub use command::*;
pub use event::Event;
use macros::fatal;
use std::{
    ffi::c_void,
    sync::mpsc::{Receiver, Sender},
};

type Subscriptions = Vec<(extern "C" fn(*const Event, *mut c_void), *mut c_void)>;

pub(crate) struct Ctx {
    subscriptions: Subscriptions,
    cmd_tx: Sender<Command>,
    cmd_rx: Option<Receiver<Command>>,
    e_tx: Option<Sender<Event>>,
    e_rx: Receiver<Event>,
}
macro_rules! cast_ctx {
    ($ctx:ident) => {{
        let $ctx = unsafe { $ctx.cast::<$crate::Ctx>().as_mut() };
        $ctx.unwrap_or_else(|| $crate::macros::fatal!("Can't read NULL ctx"))
    }};
}
pub(crate) use cast_ctx;

#[no_mangle]
pub extern "C" fn layer_shell_io_init() -> *mut c_void {
    let logger = Box::leak(Box::new(logger::StdErrLogger::new()));
    if let Err(err) = log::set_logger(logger) {
        eprintln!("Failed to set logger: {:?}", err);
    } else {
        log::set_max_level(log::LevelFilter::Trace);
    }

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<Command>();
    let (e_tx, e_rx) = std::sync::mpsc::channel::<Event>();

    let ctx = Ctx {
        subscriptions: vec![],
        cmd_tx,
        cmd_rx: Some(cmd_rx),
        e_tx: Some(e_tx),
        e_rx,
    };
    (Box::leak(Box::new(ctx)) as *mut Ctx).cast()
}

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(
    f: extern "C" fn(*const Event, *mut c_void),
    data: *mut c_void,
    ctx: *mut c_void,
) {
    cast_ctx!(ctx).subscriptions.push((f, data));
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread(ctx: *mut c_void) {
    let Some(cmd_rx) = cast_ctx!(ctx).cmd_rx.take() else {
        fatal!("layer_shell_io_spawn_thread has been called twice");
    };
    let Some(e_tx) = cast_ctx!(ctx).e_tx.take() else {
        fatal!("layer_shell_io_spawn_thread has been called twice");
    };

    std::thread::spawn(move || {
        use crate::modules::{
            app_list::AppList, control::Control, cpu::CPU, hyprland::Hyprland, memory::Memory,
            network::Network, pipewire::Pipewire, session::Session, time::Time, tray::Tray,
            weather::Weather,
        };
        use scheduler::{Config, Scheduler};

        let mut config = Config::new();
        config.add::<Control>();
        config.add::<CPU>();
        config.add::<Hyprland>();
        config.add::<Memory>();
        config.add::<Network>();
        config.add::<Pipewire>();
        config.add::<Time>();
        config.add::<Tray>();
        config.add::<Weather>();
        config.add::<AppList>();
        config.add::<Session>();

        let scheduler = Scheduler::new(config, e_tx, cmd_rx);
        scheduler.run();
    });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_poll_events(ctx: *mut c_void) {
    while let Ok(event) = cast_ctx!(ctx).e_rx.try_recv() {
        log::info!("Received event {:?}", event);

        for (sub, data) in cast_ctx!(ctx).subscriptions.iter() {
            sub(&event, *data);
        }
    }
}
