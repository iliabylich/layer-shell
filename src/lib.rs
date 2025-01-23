#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod args;
mod command;
mod dbus;
mod event;
mod fatal;
mod ffi;
mod ipc;
mod lock_channel;
mod modules;
mod scheduler;
mod subscriptions;

pub use command::Command;
pub use event::Event;

use args::parse_args;
use ipc::IPC;
use subscriptions::Subscriptions;

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(f: extern "C" fn(*const Event)) {
    Subscriptions::add(f);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() {
    pretty_env_logger::init();
    parse_args();
    IPC::set_current_process_as_main();
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
    std::thread::spawn(move || {
        use crate::modules::{
            cpu, hyprland, memory, network, pipewire, session, time, tray, weather,
        };

        pipewire::setup();
        hyprland::setup();
        network::setup();
        tray::setup();
        session::setup();

        use scheduler::Scheduler;
        let mut scheduler = Scheduler::new(40);
        scheduler.add(1_000, time::tick);
        scheduler.add(1_000, memory::tick);
        scheduler.add(1_000, cpu::tick);
        scheduler.add(120_000, weather::tick);

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
