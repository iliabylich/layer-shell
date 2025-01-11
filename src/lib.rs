#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod args;
mod command;
mod dbus;
mod event;
mod ffi;
mod global;
mod ipc;
mod modules;
mod scheduler;
mod subscriptions;

pub use command::Command;
pub use event::Event;
use global::global;

use args::parse_args;
use ipc::IPC;
use subscriptions::Subscriptions;

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(f: extern "C" fn(*const Event)) {
    Subscriptions::add(f);
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() {
    Subscriptions::setup();
    if let Err(err) = IPC::prepare() {
        log::error!("Failed to start IPC: {:?}", err);
        std::process::exit(1);
    }
    if let Err(err) = parse_args() {
        log::error!("Error while parsing args: {:?}", err);
        std::process::exit(1);
    }
    if let Err(err) = IPC::set_current_process_as_main() {
        log::error!("Failed to set current process as main in IPC: {:?}", err);
        std::process::exit(1);
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread() {
    let (etx, erx) = std::sync::mpsc::channel::<Event>();
    let (ctx, crx) = std::sync::mpsc::channel::<Command>();

    Command::set_sender(ctx);
    Event::set_sender(etx.clone());
    Event::set_receiver(erx);

    std::thread::spawn(move || {
        crate::modules::cpu::setup();
        crate::modules::pipewire::setup();
        crate::modules::hyprland::setup();
        crate::modules::app_list::setup();
        crate::modules::network::setup();

        use scheduler::Scheduler;
        let mut scheduler = Scheduler::new(40, crx);
        scheduler.add(1_000, crate::modules::time::tick);
        scheduler.add(1_000, crate::modules::memory::tick);
        scheduler.add(1_000, crate::modules::cpu::tick);
        scheduler.add(3_000, crate::modules::network::tick);
        scheduler.add(120_000, crate::modules::weather::tick);

        loop {
            scheduler.tick();
        }
    });
}

#[no_mangle]
pub extern "C" fn layer_shell_io_poll_events() {
    while let Some(event) = Event::try_recv() {
        log::info!("Received event {:?}", event);

        for f in Subscriptions::iter() {
            (f)(&event);
        }
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
