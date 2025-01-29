#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod command;
mod dbus;
mod event;
mod ffi;
mod lock_channel;
mod logger;
mod macros;
mod modules;
mod scheduler;
mod subscriptions;

pub use command::Command;
pub use event::Event;

use scheduler::Scheduler;
use subscriptions::Subscriptions;

#[no_mangle]
pub extern "C" fn layer_shell_io_init() {
    let logger = Box::leak(Box::new(logger::StdErrLogger::new()));
    if let Err(err) = log::set_logger(logger) {
        eprintln!("Failed to set logger: {:?}", err);
    } else {
        log::set_max_level(log::LevelFilter::Trace);
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(f: extern "C" fn(*const Event)) {
    Subscriptions::add(f);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
    std::thread::spawn(move || {
        use crate::modules::{
            app_list::AppList, control::Control, cpu::CPU, hyprland::Hyprland, memory::Memory,
            network::Network, pipewire::Pipewire, session::Session, time::Time, tray::Tray,
            weather::Weather,
        };

        Scheduler::init();
        let mut scheduler = Scheduler::new();

        scheduler.add::<Control>();
        scheduler.add::<CPU>();
        scheduler.add::<Hyprland>();
        scheduler.add::<Memory>();
        scheduler.add::<Network>();
        scheduler.add::<Pipewire>();
        scheduler.add::<Time>();
        scheduler.add::<Tray>();
        scheduler.add::<Weather>();
        scheduler.add::<AppList>();
        scheduler.add::<Session>();

        scheduler.start_loop();
    });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_poll_events() {
    while let Some(event) = Event::try_recv() {
        log::info!("Received event {:?}", event);
        Subscriptions::call_each(&event);
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_publish(command: Command) {
    command.send();
}
