mod command;
mod config;
mod dbus;
mod event;
mod ffi;
mod liburing;
mod logger;
mod macros;
mod modules;
mod sansio;
mod timer;
mod unix_socket;
mod user_data;

use command::Command;
use config::{Config, IOConfig};
pub use event::Event;
pub use ffi::{FFIArray, FFIString};

use crate::{
    dbus::messages::org_freedesktop_dbus::Hello,
    liburing::IoUring,
    logger::Logger,
    macros::report_and_exit,
    modules::{
        CPU, Clock, Control, ControlRequest, Hyprland, HyprlandReader, HyprlandWriter, Location,
        Memory, Module, Network, SessionDBus, Sound, SystemDBus, Tray, Weather,
    },
    sansio::{DBusQueue, Satisfy, Wants},
    timer::Timer,
    user_data::{ModuleId, UserData},
};

struct IO {
    config: Config,
    io_config: *const IOConfig,

    timer: Timer,

    session_dbus: Option<SessionDBus>,
    session_dbus_queue: DBusQueue,

    system_dbus: Option<SystemDBus>,
    system_dbus_queue: DBusQueue,

    hyprland_reader: Option<HyprlandReader>,
    hyprland_writer: Option<HyprlandWriter>,

    location: Option<Location>,
    weather: Option<Weather>,
    cpu: Option<CPU>,
    memory: Option<Memory>,
    sound: Box<Sound>,
    control: Box<Control>,
    network: Box<Network>,
    tray: Box<Tray>,

    on_event: extern "C" fn(event: *const Event),
    running: bool,
    logging_enabled: bool,
}

static mut GLOBAL_IO: *mut IO = std::ptr::null_mut();

impl IO {
    fn new(on_event: extern "C" fn(event: *const Event), logging_enabled: bool) -> Self {
        let config = Config::read().unwrap_or_else(|err| report_and_exit!("{err:?}"));
        let io_config = Box::leak(Box::new(IOConfig::from(&config)));

        let (hyprland_reader, hyprland_writer) = Hyprland::connect();

        let session_dbus_queue = DBusQueue::new();
        let system_dbus_queue = DBusQueue::new();

        let mut this = Self {
            config,
            io_config,

            timer: Timer::new(()),

            session_dbus: Some(SessionDBus::new(session_dbus_queue.clone())),
            session_dbus_queue,

            system_dbus: Some(SystemDBus::new(system_dbus_queue.clone())),
            system_dbus_queue,

            location: Some(Location::new(())),
            weather: None,
            hyprland_reader,
            hyprland_writer,
            cpu: Some(CPU::new(())),
            memory: Some(Memory::new(())),
            sound: Sound::new(),
            control: Control::new(),
            network: Network::new(),
            tray: Tray::new(),

            on_event,
            running: true,
            logging_enabled,
        };

        this.init();

        this
    }

    fn init(&mut self) {
        schedule_wanted(&mut self.timer);

        schedule_wanted(&mut self.location);
        schedule_wanted(&mut self.hyprland_reader);
        schedule_wanted(&mut self.hyprland_writer);
        schedule_wanted(&mut self.cpu);
        schedule_wanted(&mut self.memory);

        self.session_dbus_queue.push_back(&mut Hello.into());
        self.sound.init(&self.session_dbus_queue);
        self.control.init(&self.session_dbus_queue);
        self.tray.init(&self.session_dbus_queue);
        schedule_wanted(&mut self.session_dbus);

        self.system_dbus_queue.push_back(&mut Hello.into());
        self.network.init(&self.system_dbus_queue);
        schedule_wanted(&mut self.system_dbus);

        IoUring::submit_if_dirty();
    }

    fn on_control_req(&mut self, req: ControlRequest, events: &mut Vec<Event>) {
        match req {
            ControlRequest::CapsLockToggled => {
                if let Some(hyprland_writer) = &mut self.hyprland_writer {
                    hyprland_writer.enqueue_get_caps_lock();
                    schedule_wanted(hyprland_writer);
                }
                IoUring::submit_if_dirty();
            }
            ControlRequest::Exit => events.push(Event::Exit),
            ControlRequest::ReloadStyles => events.push(Event::ReloadStyles),
            ControlRequest::ToggleSessionScreen => events.push(Event::ToggleSessionScreen),
        }
    }

    fn handle_readable(&mut self) {
        if !self.running {
            return;
        }

        let mut events = vec![];

        while let Some(cqe) = IoUring::try_get_cqe() {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::from(user_data);
            let satisfy = Satisfy::from(op);

            match module_id {
                ModuleId::GeoLocation => {
                    let Ok(latlng) = self.location.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.location);
                    if let Some((lat, lng)) = latlng.flatten() {
                        self.weather = Some(Weather::new((lat, lng)));
                        schedule_wanted(&mut self.weather);
                    }
                }

                ModuleId::Weather => {
                    let Ok(_) = self.weather.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.weather);
                }

                ModuleId::HyprlandReader => {
                    let Ok(_) = self.hyprland_reader.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.hyprland_reader);
                }
                ModuleId::HyprlandWriter => {
                    let Ok(_) = self.hyprland_writer.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.hyprland_writer);
                }

                ModuleId::SessionDBus => {
                    let Ok(message) = self.session_dbus.satisfy(satisfy, res, &mut events);

                    if let Some(message) = message.flatten() {
                        self.sound
                            .on_message(&self.session_dbus_queue, &message, &mut events);
                        self.tray
                            .on_message(&self.session_dbus_queue, &message, &mut events);

                        if let Some(req) =
                            self.control.on_message(&message, &self.session_dbus_queue)
                        {
                            self.on_control_req(req, &mut events);
                        }
                    }

                    schedule_wanted(&mut self.session_dbus);
                }

                ModuleId::SystemDBus => {
                    let Ok(message) = self.system_dbus.satisfy(satisfy, res, &mut events);

                    if let Some(message) = message.flatten() {
                        self.network
                            .on_message(&self.system_dbus_queue, &message, &mut events);
                    }

                    schedule_wanted(&mut self.system_dbus);
                }

                ModuleId::CPU => {
                    let Ok(_) = self.cpu.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.cpu);
                }
                ModuleId::Memory => {
                    let Ok(_) = self.memory.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.memory);
                }
                ModuleId::Timer => {
                    let Ok(tick) = self.timer.satisfy(satisfy, res, &mut events);
                    schedule_wanted(&mut self.timer);

                    Clock::tick(&mut events);
                    self.weather.tick(tick);
                    self.cpu.tick(tick);
                    schedule_wanted(&mut self.cpu);
                    self.memory.tick(tick);
                    schedule_wanted(&mut self.memory);
                    self.sound.tick(tick, &self.session_dbus_queue);
                }
            }

            IoUring::cqe_seen(cqe);
        }

        IoUring::submit_if_dirty();

        for event in events {
            if self.logging_enabled {
                log::info!(target: "IO", "{event:?}");
            }
            (self.on_event)(&event);
        }
    }

    fn wait_readable(&mut self) {
        IoUring::submit_and_wait(1)
    }

    fn process_command(&mut self, cmd: Command) {
        if !self.running {
            return;
        }

        macro_rules! hyprctl {
            ($($arg:tt)*) => {{
                if let Some(hyprland_writer) = &mut self.hyprland_writer {
                    hyprland_writer.enqueue_dispatch(format!($($arg)*), );
                    schedule_wanted(hyprland_writer);
                }
                IoUring::submit_if_dirty();
            }};
        }
        match cmd {
            Command::GoToWorkspace { workspace } => {
                hyprctl!("workspace {}", workspace)
            }
            Command::Lock => {
                hyprctl!("exec {}", self.config.lock)
            }
            Command::Reboot => {
                hyprctl!("exec {}", self.config.reboot)
            }
            Command::Shutdown => {
                hyprctl!("exec {}", self.config.shutdown)
            }
            Command::Logout => {
                hyprctl!("exit")
            }
            Command::SpawnWiFiEditor => {
                hyprctl!("exec {}", self.config.edit_wifi)
            }
            Command::SpawnBluetoothEditor => {
                hyprctl!("exec {}", self.config.edit_bluetooth)
            }
            Command::SpawnSystemMonitor => {
                hyprctl!("exec {}", self.config.open_system_monitor)
            }
            Command::ChangeTheme => {
                hyprctl!("exec {}", self.config.change_theme)
            }

            Command::TriggerTray { uuid } => {
                self.tray.trigger(&uuid, &self.session_dbus_queue);
                IoUring::submit_if_dirty();
            }
        }
    }

    fn deinit(&mut self) {
        self.running = false;
        IoUring::deinit();
    }
}

fn io_mut() -> &'static mut IO {
    unsafe { GLOBAL_IO.as_mut() }
        .unwrap_or_else(|| report_and_exit!("IO is not initialized. Call io_init() first."))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init(on_event: extern "C" fn(event: *const Event), logging_enabled: bool) {
    if unsafe { !GLOBAL_IO.is_null() } {
        report_and_exit!("io_init() called while IO is already initialized");
    }

    Logger::init();

    rustls_openssl::default_provider()
        .install_default()
        .unwrap_or_else(|_| report_and_exit!("failed to install OpenSSL CryptoProvider"));
    IoUring::init(10, 0);
    unsafe {
        GLOBAL_IO = Box::into_raw(Box::new(IO::new(on_event, logging_enabled)));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit() {
    if unsafe { GLOBAL_IO.is_null() } {
        report_and_exit!("io_deinit() called while IO is not initialized");
    }

    unsafe {
        (*GLOBAL_IO).deinit();
        drop(Box::from_raw(GLOBAL_IO));
        GLOBAL_IO = std::ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable() {
    io_mut().handle_readable();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable() {
    io_mut().wait_readable();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    IoUring::as_raw_fd()
}

#[unsafe(no_mangle)]
pub extern "C" fn io_get_config() -> *const IOConfig {
    io_mut().io_config
}

fn process_command(cmd: Command) {
    io_mut().process_command(cmd);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_hyprland_go_to_workspace(workspace: usize) {
    process_command(Command::GoToWorkspace { workspace });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_lock() {
    process_command(Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() {
    process_command(Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() {
    process_command(Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() {
    process_command(Command::Logout);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(uuid: *const std::ffi::c_char) {
    process_command(Command::TriggerTray {
        uuid: FFIString::from(uuid).into(),
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() {
    process_command(Command::SpawnWiFiEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() {
    process_command(Command::SpawnBluetoothEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() {
    process_command(Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme() {
    process_command(Command::ChangeTheme);
}

fn schedule_wanted<T>(module: &mut T)
where
    T: Module,
{
    match module.wants() {
        Wants::Socket { domain, r#type } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_socket(domain, r#type, 0, 0);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Socket));
        }
        Wants::Connect { fd, addr, addrlen } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_connect(fd, addr, addrlen);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Connect));
        }
        Wants::Read { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_read(fd, buf, len);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Read));
        }
        Wants::Write { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_write(fd, buf, len);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Write));
        }
        Wants::ReadWrite {
            fd,
            readbuf,
            readlen,
            writebuf,
            writelen,
        } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_read(fd, readbuf, readlen);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Read));

            let mut sqe = IoUring::get_sqe();
            sqe.prep_write(fd, writebuf, writelen);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Write));
        }
        Wants::OpenAt {
            dfd,
            path,
            flags,
            mode,
        } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_openat(dfd, path, flags, mode);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::OpenAt));
        }
        Wants::Close { fd } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_close(fd);
            sqe.set_user_data(UserData::new(T::MODULE_ID, Satisfy::Close));
        }
        Wants::Nothing => {}
    }
}
