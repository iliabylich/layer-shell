#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod args;
mod command;
mod dbus;
mod event;
mod fatal;
mod ffi;
mod global;
mod ipc;
mod lock_channel;
mod modules;
mod scheduler;
mod subscriptions;

pub use command::Command;
pub use event::Event;
use global::global;

use args::parse_args;
use fatal::fatal;
use ipc::IPC;
use subscriptions::Subscriptions;

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(f: extern "C" fn(*const Event)) {
    Subscriptions::add(f);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() {
    if let Err(err) = parse_args() {
        fatal!("Error while parsing args: {:?}", err);
    }
    if let Err(err) = IPC::set_current_process_as_main() {
        fatal!("Failed to set current process as main in IPC: {:?}", err);
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
    std::thread::spawn(move || {
        use crate::modules::{cpu, hyprland, memory, network, pipewire, time, weather};

        cpu::setup();
        pipewire::setup();
        hyprland::setup();
        network::setup();

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

#[no_mangle]
pub extern "C" fn layer_shell_io_init_logger() {
    pretty_env_logger::init();
}
