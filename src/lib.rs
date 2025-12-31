// mod command;
// mod config;
mod event;
// mod main_loop;
mod array_writer;
mod clock;
mod ffi;
mod https;
mod hyprland;
mod liburing;
mod timerfd;
mod weather;

use anyhow::Result;
// use command::Command;
pub use event::Event;
pub use ffi::{CArray, CString};
// use main_loop::MainLoop;
use std::{ffi::c_void, os::fd::AsRawFd};
// use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

// use crate::config::{Config, IOConfig};

thread_local! {
//     static CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) };
//     static IO_CONFIG: RefCell<Option<IOConfig>> = const { RefCell::new(None) };
}

use crate::{
    clock::Clock, hyprland::Hyprland, liburing::Actor, liburing::IoUring, timerfd::Timerfd,
    weather::Weather,
};

#[repr(u64)]
enum UserData {
    GetLocationSocket = 1,
    GetLocationConnect,
    GetLocationRead,
    GetLocationWrite,
    GetLocationClose,

    GetWeatherSocket,
    GetWeatherConnect,
    GetWeatherRead,
    GetWeatherWrite,
    GetWeatherClose,

    HyprlandReaderRead,

    HyprlandWriterSocket,
    HyprlandWriterConnect,
    HyprlandWriterWrite,
    HyprlandWriterRead,
    HyprlandWriterClose,

    TimerfdRead,
}

struct IO {
    ring: IoUring,
    timer: Timerfd,

    clock: Clock,
    weather: Weather,
    hyprland: Hyprland,
    on_event: fn(event: Event),
}

impl IO {
    fn new(on_event: fn(event: Event)) -> Self {
        let mut ring = IoUring::new(10, 0).unwrap();
        let mut timer = Timerfd::new(UserData::TimerfdRead as u64).unwrap();

        let mut clock = Clock::new();
        let mut weather = Weather::new().unwrap();
        let mut hyprland = Hyprland::new().unwrap();

        let mut events = vec![];

        let drained = timer.drain(&mut ring).unwrap();
        assert!(drained);

        let drained = clock.drain_to_end(&mut ring, &mut events).unwrap();
        assert!(!drained);

        let drained = weather.drain_to_end(&mut ring, &mut events).unwrap();
        assert!(drained);

        let drained = hyprland.drain_to_end(&mut ring, &mut events).unwrap();
        assert!(drained);

        for event in events {
            (on_event)(event);
        }

        ring.submit().unwrap();

        IO {
            ring,
            timer,

            clock,
            weather,
            hyprland,
            on_event,
        }
    }

    fn process(&mut self) -> Result<()> {
        let mut drained = false;
        let mut events = vec![];

        while let Some(cqe) = self.ring.try_get_cqe().unwrap() {
            if let Some(tick) = self.timer.feed(cqe).unwrap() {
                self.clock.on_tick(tick);
            }
            drained |= self.timer.drain(&mut self.ring).unwrap();

            self.clock.feed(&mut self.ring, cqe, &mut events).unwrap();
            drained |= self
                .clock
                .drain_to_end(&mut self.ring, &mut events)
                .unwrap();

            self.weather.feed(&mut self.ring, cqe, &mut events).unwrap();
            drained |= self
                .weather
                .drain_to_end(&mut self.ring, &mut events)
                .unwrap();

            self.hyprland
                .feed(&mut self.ring, cqe, &mut events)
                .unwrap();
            drained |= self
                .hyprland
                .drain_to_end(&mut self.ring, &mut events)
                .unwrap();

            self.ring.cqe_seen(cqe);
        }

        for event in events {
            (self.on_event)(event);
        }

        if drained {
            self.ring.submit().unwrap()
        }

        Ok(())
    }

    fn wait(&mut self) -> Result<()> {
        self.ring.submit_and_wait(1)
    }
}

pub fn io_init(on_event: fn(event: Event)) -> *mut c_void {
    env_logger::init();

    // let (etx, erx) = tokio::sync::mpsc::unbounded_channel::<Event>();
    // let (ctx, crx) = tokio::sync::mpsc::unbounded_channel::<Command>();
    // let config = match Config::read() {
    //     Ok(config) => config,
    //     Err(err) => {
    //         log::error!("{err:?}");
    //         std::process::exit(1);
    //     }
    // };
    // let io_config = IOConfig::from(&config);

    // let (pipe_reader, pipe_writer) = match std::io::pipe() {
    //     Ok(pair) => pair,
    //     Err(err) => {
    //         log::error!("{err:?}");
    //         std::process::exit(1);
    //     }
    // };

    // let fd = pipe_reader.as_raw_fd();

    // ETX.set(Some(etx));
    // ERX.set(Some(erx));
    // CTX.set(Some(ctx));
    // CRX.set(Some(crx));
    // CONFIG.set(Some(config));
    // IO_CONFIG.set(Some(io_config));
    // PIPE_WRITER.set(Some(pipe_writer));
    // PIPE_READER.set(Some(pipe_reader));

    (Box::leak(Box::new(IO::new(on_event))) as *mut IO).cast()
}

pub fn io_process(io: *mut c_void) {
    let io = unsafe { io.cast::<IO>().as_mut() }.unwrap();

    if let Err(err) = io.process() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

pub fn io_wait(io: *mut c_void) {
    let io = unsafe { io.cast::<IO>().as_mut() }.unwrap();
    if let Err(err) = io.wait() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

// pub fn io_take_ctx() -> (
//     UnboundedSender<Event>,
//     UnboundedReceiver<Command>,
//     Config,
//     PipeWriter,
// ) {
//     let Some(etx) = ETX.take() else {
//         log::error!("ETX is not set, did you call io_init()?");
//         std::process::exit(1);
//     };
//     let Some(crx) = CRX.take() else {
//         log::error!("CRX is not set, did you call io_init()?");
//         std::process::exit(1);
//     };
//     let Some(config) = CONFIG.take() else {
//         log::error!("CONFIG is not set, did you call io_init()?");
//         std::process::exit(1);
//     };
//     let Some(pipe_writer) = PIPE_WRITER.take() else {
//         log::error!("PIPE_WRITER is not set, did you call io_init()?");
//         std::process::exit(1);
//     };
//     (etx, crx, config, pipe_writer)
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_thread() {
//     let ring = liburing::IoUring::new(10, 0).unwrap();

//     let (etx, crx, config, pipe_writer) = io_take_ctx();

//     let handle = std::thread::spawn(move || {
//         if let Err(err) = io_run_in_place(config, etx, crx, pipe_writer) {
//             log::error!("IO thread has crashed: {:?}", err);
//         }
//     });

//     THREAD_HANDLE.set(Some(handle));
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_config() -> *const IOConfig {
//     IO_CONFIG.with(|config| {
//         if let Some(config) = config.borrow().as_ref() {
//             config as *const IOConfig
//         } else {
//             log::error!("IO_CONFIG is not set, did you call io_init() ?");
//             std::process::exit(1)
//         }
//     })
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_poll_events() -> CArray<Event> {
//     let mut out = vec![];

//     fn try_recv() -> Result<Event> {
//         ERX.with_borrow_mut(|erx| {
//             let erx = erx.as_mut().unwrap_or_else(|| {
//                 log::error!("ERX is not set, did you call io_init()?");
//                 std::process::exit(1);
//             });
//             erx.try_recv().context("recv() failed")
//         })
//     }

//     while let Ok(event) = try_recv() {
//         out.push(event);
//     }
//     out.into()
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_drop_events(events: CArray<Event>) {
//     drop(events)
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_finalize() {
//     fn try_io_finalize() -> Result<()> {
//         send_command(Command::FinishIoThread);
//         log::info!("Waiting for IO thread to finish...");
//         THREAD_HANDLE
//             .take()
//             .context("THREAD_HANDLE is not set, did you call io_init()")?
//             .join()
//             .map_err(|_| anyhow::anyhow!("io thread is not running, can't stop gracefully"))?;
//         log::info!("IO thread has finished...");
//         Ok(())
//     }

//     if let Err(err) = try_io_finalize() {
//         log::error!("{err:?}");
//         std::process::exit(1);
//     }
// }

// fn send_command(cmd: Command) {
//     CTX.with_borrow_mut(|ctx| {
//         let ctx = ctx.as_mut().unwrap_or_else(|| {
//             log::error!("no CTX, did you call io_init()?");
//             std::process::exit(1);
//         });

//         if ctx.send(cmd).is_err() {
//             log::error!("failed to send Command, channel is closed");
//             std::process::exit(1);
//         }
//     });
// }

// #[unsafe(no_mangle)]
// pub extern "C" fn io_hyprland_go_to_workspace(workspace: usize) {
//     send_command(Command::HyprlandGoToWorkspace { workspace });
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_lock() {
//     send_command(Command::Lock);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_reboot() {
//     send_command(Command::Reboot);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_shutdown() {
//     send_command(Command::Shutdown);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_logout() {
//     send_command(Command::Logout);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_trigger_tray(uuid: *const std::ffi::c_char) {
//     send_command(Command::TriggerTray {
//         uuid: CString::from(uuid).into(),
//     });
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_wifi_editor() {
//     send_command(Command::SpawnWiFiEditor);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_bluetooh_editor() {
//     send_command(Command::SpawnBluetoothEditor);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_system_monitor() {
//     send_command(Command::SpawnSystemMonitor);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_change_theme() {
//     send_command(Command::ChangeTheme);
// }
