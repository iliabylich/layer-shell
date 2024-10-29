use layer_shell_utils::global;

mod actors;
mod args;
mod command;
mod event;
mod ipc;

pub use command::Command;
pub use event::{App, AppIcon, Event};
pub use ipc::on_sigusr1;

use args::parse_args;
use ipc::IPC;
use std::sync::mpsc::{channel, Receiver, Sender};

global!(COMMAND_SENDER, Sender<Command>);
global!(EVENT_RECEIVER, Receiver<Event>);
global!(EVENT_SENDER, Sender<Event>);
global!(SUBSCRIPTIONS, Vec<fn(&Event)>);

pub fn subscribe(f: fn(&Event)) {
    SUBSCRIPTIONS::get().push(f);
}

pub fn init() {
    pretty_env_logger::init();
    SUBSCRIPTIONS::set(vec![]);
    if let Err(err) = parse_args() {
        log::error!("Error while parsing args: {}", err);
        std::process::exit(1);
    }
    if let Err(err) = IPC::init() {
        log::error!("Failed to start IPC: {}", err);
        std::process::exit(1);
    }
}

pub fn spawn_all() {
    let (etx, erx) = channel::<Event>();
    let (ctx, crx) = channel::<Command>();

    COMMAND_SENDER::set(ctx);
    EVENT_RECEIVER::set(erx);
    EVENT_SENDER::set(etx.clone());

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::join!(
                // command processing actor
                command::start_processing(crx),
                // and all models
                actors::memory::spawn(etx.clone()),
                actors::cpu::spawn(etx.clone()),
                actors::time::spawn(etx.clone()),
                actors::hyprland::spawn(etx.clone()),
                actors::app_list::spawn(etx.clone()),
                actors::output_sound::spawn(etx.clone()),
                actors::session::spawn(etx.clone()),
                actors::weather::spawn(etx.clone()),
                actors::network_manager::wifi_status::spawn(etx.clone()),
                actors::network_manager::network_list::spawn(etx.clone()),
            );
        });
    });
}

pub fn poll_events() {
    while let Ok(event) = EVENT_RECEIVER::get().try_recv() {
        log::info!("Received event {:?}", event);

        for f in SUBSCRIPTIONS::get().iter() {
            (f)(&event);
        }
    }
}

pub fn publish(c: Command) {
    if let Err(err) = COMMAND_SENDER::get().send(c) {
        log::error!("failed to publish event: {}", err);
    }
}

pub(crate) fn publish_event(e: Event) {
    if let Err(err) = EVENT_SENDER::get().send(e) {
        log::error!("failed to publish event: {}", err);
    }
}
