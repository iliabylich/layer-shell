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
use macros::{cast_ctx_mut, cast_ctx_ref};

type Subscriptions = Vec<(
    extern "C" fn(*const Event, *mut std::ffi::c_void),
    *mut std::ffi::c_void,
)>;

#[repr(C)]
pub struct Ctx {
    subscriptions: *mut std::ffi::c_void,
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() -> Ctx {
    let logger = Box::leak(Box::new(logger::StdErrLogger::new()));
    if let Err(err) = log::set_logger(logger) {
        eprintln!("Failed to set logger: {:?}", err);
    } else {
        log::set_max_level(log::LevelFilter::Trace);
    }

    Ctx {
        subscriptions: (Box::leak(Box::new(vec![])) as *mut Subscriptions).cast(),
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(
    f: extern "C" fn(*const Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
    subscriptions: *mut std::ffi::c_void,
) {
    let subscriptions = cast_ctx_mut!(subscriptions, Subscriptions);
    subscriptions.push((f, data));
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
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

        let scheduler = Scheduler::new(config);
        scheduler.run();
    });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_poll_events(subscriptions: *const std::ffi::c_void) {
    while let Some(event) = Event::try_recv() {
        log::info!("Received event {:?}", event);
        let subscriptions = cast_ctx_ref!(subscriptions, Subscriptions);

        for (sub, data) in subscriptions.iter() {
            sub(&event, *data);
        }
    }
}
