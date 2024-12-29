#![allow(clippy::type_complexity)]
#![allow(clippy::upper_case_acronyms)]

mod actors;
mod args;
mod command;
mod dbus;
mod event;
mod ffi;
mod global;
mod ipc;
mod modules;

pub use command::Command;
pub use event::Event;
pub(crate) use global::global;
pub use ipc::on_sigusr1;

use args::parse_args;
use ipc::IPC;
use std::sync::mpsc::{channel, Receiver, Sender};

global!(COMMAND_SENDER, Sender<Command>);
global!(EVENT_RECEIVER, Receiver<Event>);
global!(EVENT_SENDER, Sender<Event>);
global!(SUBSCRIPTIONS, Vec<extern "C" fn(*const Event)>);

#[no_mangle]
pub extern "C" fn subscribe(f: extern "C" fn(*const Event)) {
    SUBSCRIPTIONS::get().push(f);
}

#[no_mangle]
pub extern "C" fn init() {
    SUBSCRIPTIONS::set(vec![]);
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
pub extern "C" fn spawn_thread() {
    let (etx, erx) = channel::<Event>();
    let (ctx, crx) = channel::<Command>();

    COMMAND_SENDER::set(ctx);
    EVENT_RECEIVER::set(erx);
    EVENT_SENDER::set(etx.clone());

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                log::error!("failed to spawn tokio: {:?}", err);
                std::process::exit(1);
            }
        };

        rt.block_on(async {
            tokio::join!(
                // command processing actor
                command::start_processing(crx),
                // and all models
                actors::spawn_all(etx),
            );
        });
    });
}

#[no_mangle]
pub extern "C" fn poll_events() {
    while let Ok(event) = EVENT_RECEIVER::get().try_recv() {
        log::info!("Received event {:?}", event);

        for f in SUBSCRIPTIONS::get().iter() {
            (f)(&event);
        }
    }
}

#[no_mangle]
pub extern "C" fn publish(c: Command) {
    if let Err(err) = COMMAND_SENDER::get().send(c) {
        log::error!("failed to publish event: {:?}", err);
    }
}

pub(crate) fn publish_event(e: Event) {
    if let Err(err) = EVENT_SENDER::get().send(e) {
        log::error!("failed to publish event: {:?}", err);
    }
}

#[no_mangle]
pub extern "C" fn init_logger() {
    pretty_env_logger::init();
}

#[no_mangle]
pub static mut MAIN_CSS: *const u8 = concat!(include_str!("../ui/main.css"), "\0").as_ptr();

pub mod icons {
    use crate::ffi::CBytes;

    macro_rules! icon {
        ($path:literal) => {{
            CBytes::new(include_bytes!($path))
        }};
    }

    #[no_mangle]
    pub static mut FOGGY: CBytes = icon!("../icons/foggy.png");
    #[no_mangle]
    pub static mut QUESTION_MARK: CBytes = icon!("../icons/question_mark.png");
    #[no_mangle]
    pub static mut SUNNY: CBytes = icon!("../icons/sunny.png");
    #[no_mangle]
    pub static mut PARTLY_CLOUDY: CBytes = icon!("../icons/partly_cloudy.png");
    #[no_mangle]
    pub static mut RAINY: CBytes = icon!("../icons/rainy.png");
    #[no_mangle]
    pub static mut THUNDERSTORM: CBytes = icon!("../icons/thunderstorm.png");
    #[no_mangle]
    pub static mut POWER: CBytes = icon!("../icons/power.png");
    #[no_mangle]
    pub static mut SNOWY: CBytes = icon!("../icons/snowy.png");
    #[no_mangle]
    pub static mut WIFI: CBytes = icon!("../icons/wifi.png");
}
