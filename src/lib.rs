// mod command;
// mod config;
mod event;
// mod main_loop;
mod dbus;
mod ffi;
mod https;
mod liburing;
mod modules;
mod timerfd;
mod user_data;

use anyhow::Result;
// use command::Command;
pub use event::Event;
pub use ffi::{CArray, CString};
use std::{ffi::c_void, os::fd::AsRawFd};

// use crate::config::{Config, IOConfig};

thread_local! {
//     static CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) };
//     static IO_CONFIG: RefCell<Option<IOConfig>> = const { RefCell::new(None) };
}

use crate::{
    dbus::{BuiltinDBusMessage, DBus, messages::org_freedesktop_dbus::Hello},
    liburing::IoUring,
    modules::{CPU, Clock, Control, ControlRequest, Hyprland, Memory, Sound, Weather},
    timerfd::Timerfd,
    user_data::UserData,
};

struct IO {
    ring: IoUring,

    timer: Box<Timerfd>,
    session_dbus: Box<DBus>,
    system_dbus: Box<DBus>,

    weather: Box<Weather>,
    hyprland: Box<Hyprland>,
    cpu: Box<CPU>,
    memory: Box<Memory>,

    sound: Box<Sound>,
    control: Box<Control>,

    on_event: extern "C" fn(event: Event),
}

impl IO {
    fn try_new(on_event: extern "C" fn(event: Event)) -> Result<Self> {
        let mut this = Self {
            ring: IoUring::new(10, 0)?,

            timer: Timerfd::new()?,
            session_dbus: DBus::new_session()?,
            system_dbus: DBus::new_system()?,

            weather: Weather::new()?,
            hyprland: Hyprland::new()?,
            cpu: CPU::new()?,
            memory: Memory::new()?,

            sound: Sound::new(),
            control: Control::new(),

            on_event,
        };

        this.init()?;

        Ok(this)
    }

    fn init(&mut self) -> Result<()> {
        let drained = self.timer.drain(&mut self.ring)?;
        assert!(drained);

        self.weather.drain(&mut self.ring)?;
        self.hyprland.drain(&mut self.ring)?;
        self.cpu.drain(&mut self.ring)?;
        self.memory.drain(&mut self.ring)?;

        self.session_dbus.enqueue(&mut Hello.into())?;
        self.sound.init(&mut self.session_dbus)?;
        self.control.init(&mut self.session_dbus)?;
        self.session_dbus.drain(&mut self.ring)?;

        self.system_dbus.enqueue(&mut Hello.into())?;

        self.ring.submit()?;

        Ok(())
    }

    fn new(on_event: extern "C" fn(event: Event)) -> Self {
        Self::try_new(on_event).unwrap_or_else(|err| {
            eprintln!("{err:?}");
            std::process::exit(1);
        })
    }

    fn from_raw(ptr: *mut c_void) -> &'static mut Self {
        unsafe { ptr.cast::<Self>().as_mut() }.unwrap_or_else(|| {
            eprintln!("NULL IO pointer given to IO::from_raw()");
            std::process::exit(1);
        })
    }

    fn try_handle_timer(&mut self, user_data: UserData) -> Result<bool> {
        let mut events = vec![];

        if let Some(tick) = self.timer.feed(user_data)? {
            Clock::on_tick(tick, &mut events);
            self.weather.on_tick(tick)?;
            self.cpu.on_tick(tick)?;
            self.memory.on_tick(tick)?;
        }
        let drained = self.timer.drain(&mut self.ring)?;

        for event in events {
            (self.on_event)(event);
        }

        Ok(drained)
    }

    fn try_handle_actors(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        let mut drained = false;

        drained |= {
            self.weather.feed(user_data, res, events)?;
            self.weather.drain(&mut self.ring)?
        };
        drained |= {
            self.hyprland.feed(user_data, res, events)?;
            self.hyprland.drain(&mut self.ring)?
        };
        drained |= {
            self.cpu.feed(user_data, res, events)?;
            self.cpu.drain(&mut self.ring)?
        };
        drained |= {
            self.memory.feed(user_data, res, events)?;
            self.memory.drain(&mut self.ring)?
        };

        Ok(drained)
    }

    fn on_control_req(&mut self, req: ControlRequest) -> Result<()> {
        println!("Got control request: {req:?}");
        Ok(())
    }

    fn try_handle_session_dbus(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        if let Some(message) = self.session_dbus.feed(user_data, res)? {
            if let Ok(message) = BuiltinDBusMessage::try_from(&message) {
                self.sound.on_builtin_message(&message, events)?;
                self.control
                    .on_builtin_message(&message, &mut self.session_dbus)?;
            } else {
                self.sound.on_unknown_message(&message, events)?;
                if let Some(control_req) = self
                    .control
                    .on_unknown_message(&message, &mut self.session_dbus)?
                {
                    self.on_control_req(control_req)?;
                }
            }
        }
        self.session_dbus.drain(&mut self.ring)
    }

    fn try_handle_system_dbus(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        if let Some(message) = self.system_dbus.feed(user_data, res)? {
            if let Ok(message) = BuiltinDBusMessage::try_from(&message) {
            } else {
            }
        }
        self.session_dbus.drain(&mut self.ring)
    }

    fn try_handle_readable(&mut self) -> Result<()> {
        let mut drained = false;
        let mut events = vec![];

        while let Some(cqe) = self.ring.try_get_cqe()? {
            let res = cqe.res();
            let (user_data, _request_id) = UserData::from_u64(cqe.user_data(), res);

            drained |= self.try_handle_timer(user_data)?;
            drained |= self.try_handle_actors(user_data, res, &mut events)?;
            drained |= self.try_handle_session_dbus(user_data, res, &mut events)?;
            drained |= self.try_handle_system_dbus(user_data, res, &mut events)?;

            self.ring.cqe_seen(cqe);
        }

        for event in events {
            (self.on_event)(event);
        }

        if drained {
            self.ring.submit()?
        }

        Ok(())
    }

    fn handle_readable(&mut self) {
        self.try_handle_readable().unwrap_or_else(|err| {
            eprintln!("{err:?}");
            std::process::exit(1);
        })
    }

    fn try_wait_readable(&mut self) -> Result<()> {
        self.ring.submit_and_wait(1)
    }

    fn wait_readable(&mut self) {
        self.try_wait_readable().unwrap_or_else(|err| {
            eprintln!("{err:?}");
            std::process::exit(1);
        })
    }
}

pub fn io_init(on_event: extern "C" fn(event: Event)) -> *mut c_void {
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

pub fn io_handle_readable(io: *mut c_void) {
    IO::from_raw(io).handle_readable();
}

pub fn io_wait_readable(io: *mut c_void) {
    IO::from_raw(io).wait_readable();
}

pub fn io_as_raw_fd(io: *mut c_void) -> i32 {
    IO::from_raw(io).ring.as_raw_fd()
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
