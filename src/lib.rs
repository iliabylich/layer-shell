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
use ffi::CString;
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
        scheduler.add(50, crate::modules::pipewire::tick);
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

#[no_mangle]
pub extern "C" fn layer_shell_io_main_css() -> CString {
    let home = std::env::var("HOME").unwrap();

    let theme_filepath = format!("{}/.theme.css", home);
    let theme = std::fs::read_to_string(theme_filepath).unwrap_or_default();
    let builtin = include_str!("../main.css");
    let css = format!("{}\n{}", theme, builtin);

    css.into()
}

pub mod icons {
    use crate::ffi::CBytes;

    macro_rules! icon {
        ($path:literal) => {{
            CBytes::new(include_bytes!($path))
        }};
    }

    #[no_mangle]
    pub static mut FOGGY_ICON_BYTES: CBytes = icon!("../icons/foggy.png");
    #[no_mangle]
    pub static mut QUESTION_MARK_ICON_BYTES: CBytes = icon!("../icons/question_mark.png");
    #[no_mangle]
    pub static mut SUNNY_ICON_BYTES: CBytes = icon!("../icons/sunny.png");
    #[no_mangle]
    pub static mut PARTLY_CLOUDY_ICON_BYTES: CBytes = icon!("../icons/partly_cloudy.png");
    #[no_mangle]
    pub static mut RAINY_ICON_BYTES: CBytes = icon!("../icons/rainy.png");
    #[no_mangle]
    pub static mut THUNDERSTORM_ICON_BYTES: CBytes = icon!("../icons/thunderstorm.png");
    #[no_mangle]
    pub static mut POWER_ICON_BYTES: CBytes = icon!("../icons/power.png");
    #[no_mangle]
    pub static mut SNOWY_ICON_BYTES: CBytes = icon!("../icons/snowy.png");
    #[no_mangle]
    pub static mut WIFI_ICON_BYTES: CBytes = icon!("../icons/wifi.png");
}
