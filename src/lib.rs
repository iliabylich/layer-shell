#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod command;
mod dbus;
mod event;
mod ffi;
mod lock_channel;
mod macros;
mod modules;
mod scheduler;
mod subscriptions;

pub use command::Command;
pub use event::Event;

use scheduler::Scheduler;
use subscriptions::Subscriptions;

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(f: extern "C" fn(*const Event)) {
    Subscriptions::add(f);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() {
    env_logger::init();
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
    std::thread::spawn(move || {
        use crate::modules::{
            control, cpu, hyprland, memory, network, pipewire, time, tray, weather,
        };

        let mut scheduler = Scheduler::new(40);

        scheduler.add_once("control", control::setup);
        scheduler.add("cpu", 1_000, cpu::tick);
        scheduler.add_once("hyprland", hyprland::setup);
        scheduler.add("memory", 1_000, memory::tick);
        scheduler.add_once("network", network::setup);
        scheduler.add_once("pipewire", pipewire::setup);
        scheduler.add("time", 1_000, time::tick);
        scheduler.add_once("tray", tray::setup);
        scheduler.add("weather", 120_000, weather::tick);

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
