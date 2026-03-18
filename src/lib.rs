mod command;
mod config;
mod dbus;
mod event;
mod event_queue;
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
pub use ffi::FFIArray;

use crate::{
    event_queue::EventQueue,
    ffi::ShortString,
    liburing::IoUring,
    logger::Logger,
    macros::report_and_exit,
    modules::{
        CPU, Clock, Control, ControlRequest, Hyprland, HyprlandQueue, HyprlandReader,
        HyprlandWriter, Location, Memory, Network, SessionDBus, Sound, SystemDBus, Tray, Weather,
    },
    sansio::{DBusQueue, Satisfy, Wants},
    timer::Timer,
    user_data::{ModuleId, UserData},
};

struct IO {
    config: Config,
    io_config: *const IOConfig,
    events: EventQueue,

    timer: Timer,
    clock: Clock,

    session_dbus: Option<SessionDBus>,
    sound: Sound,
    control: Control,
    tray: Tray,
    system_dbus: Option<SystemDBus>,
    network: Network,
    hyprland_reader: Option<HyprlandReader>,
    hyprland_writer: Option<HyprlandWriter>,
    hyprland_queue: HyprlandQueue,

    location: Option<Location>,
    weather: Option<Weather>,
    cpu: Option<CPU>,
    memory: Option<Memory>,

    on_event: extern "C" fn(event: *const Event),
    running: bool,
    logging_enabled: bool,
}

static mut GLOBAL_IO: *mut IO = std::ptr::null_mut();

macro_rules! schedule {
    ($module:expr) => {{
        let wants = $module.wants();
        let module_id = $module.module_id();
        schedule_wanted(wants, module_id)
    }};
}
macro_rules! schedule_opt {
    ($module:expr) => {
        if let Some(module) = &mut $module {
            schedule!(module)
        }
    };
}

impl IO {
    fn new(on_event: extern "C" fn(event: *const Event), logging_enabled: bool) -> Self {
        let config = Config::read().unwrap_or_else(|err| report_and_exit!("{err:?}"));
        let io_config = Box::leak(Box::new(IOConfig::from(&config)));
        let events = EventQueue::new();

        let (hyprland_reader, hyprland_writer, hyprland_queue) = Hyprland::connect(events.clone());

        let session_dbus_queue = DBusQueue::new();
        let system_dbus_queue = DBusQueue::new();

        let mut this = Self {
            config,
            io_config,
            events: events.clone(),

            timer: Timer::new(),
            clock: Clock::new(events.clone()),

            session_dbus: Some(SessionDBus::new(session_dbus_queue.clone())),
            sound: Sound::new(events.clone(), session_dbus_queue.clone()),
            control: Control::new(session_dbus_queue.clone()),
            tray: Tray::new(events.clone(), session_dbus_queue.clone()),
            system_dbus: Some(SystemDBus::new(system_dbus_queue.clone())),
            network: Network::new(events.clone(), system_dbus_queue.clone()),
            hyprland_reader,
            hyprland_writer,
            hyprland_queue,

            location: Some(Location::new()),
            weather: None,
            cpu: Some(CPU::new(events.clone())),
            memory: Some(Memory::new(events.clone())),

            on_event,
            running: true,
            logging_enabled,
        };

        this.init();

        this
    }

    fn init(&mut self) {
        schedule!(self.timer);

        schedule_opt!(self.location);
        schedule_opt!(self.hyprland_reader);
        schedule_opt!(self.hyprland_writer);
        schedule_opt!(self.cpu);
        schedule_opt!(self.memory);

        self.sound.init();
        self.control.init();
        self.tray.init();
        schedule_opt!(self.session_dbus);

        self.network.init();
        schedule_opt!(self.system_dbus);

        IoUring::submit_if_dirty();
    }

    fn on_control_req(&mut self, req: ControlRequest) {
        match req {
            ControlRequest::CapsLockToggled => {
                self.hyprland_queue.enqueue_get_caps_lock();
                schedule_opt!(self.hyprland_writer);
            }
            ControlRequest::Exit => self.events.push_back(Event::Exit),
            ControlRequest::ReloadStyles => self.events.push_back(Event::ReloadStyles),
            ControlRequest::ToggleSessionScreen => {
                self.events.push_back(Event::ToggleSessionScreen)
            }
        }
    }

    fn handle_readable(&mut self) {
        if !self.running {
            return;
        }

        while let Some(cqe) = IoUring::try_get_cqe() {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::from(user_data);
            let satisfy = Satisfy::from(op);

            macro_rules! satisfy {
                ($module:expr) => {
                    $module.satisfy(satisfy, res)
                };
            }
            macro_rules! satisfy_opt {
                ($module:expr) => {
                    if let Some(module) = &mut $module {
                        match satisfy!(module) {
                            Ok(value) => value,
                            Err(err) => {
                                log::error!("Module {:?} has crashed: {err:?}", module.module_id());
                                $module = None;
                                Default::default()
                            }
                        }
                    } else {
                        Default::default()
                    }
                };
            }

            match module_id {
                ModuleId::GeoLocation => {
                    let latlng = satisfy_opt!(self.location);
                    schedule_opt!(self.location);
                    if let Some((lat, lng)) = latlng {
                        self.weather = Some(Weather::new(lat, lng, self.events.clone()));
                        schedule_opt!(self.weather);
                    }
                }

                ModuleId::Weather => {
                    satisfy_opt!(self.weather);
                    schedule_opt!(self.weather);
                }

                ModuleId::HyprlandReader => {
                    satisfy_opt!(self.hyprland_reader);
                    schedule_opt!(self.hyprland_reader);
                }
                ModuleId::HyprlandWriter => {
                    satisfy_opt!(self.hyprland_writer);
                    schedule_opt!(self.hyprland_writer);
                }

                ModuleId::SessionDBus => {
                    let message = satisfy_opt!(self.session_dbus);

                    if let Some(message) = message {
                        self.sound.on_message(message);
                        self.tray.on_message(message);

                        if let Some(req) = self.control.on_message(message) {
                            self.on_control_req(req);
                        }
                    }

                    schedule_opt!(self.session_dbus);
                }

                ModuleId::SystemDBus => {
                    let message = satisfy_opt!(self.system_dbus);

                    if let Some(message) = message {
                        self.network.on_message(message);
                    }

                    schedule_opt!(self.system_dbus);
                }

                ModuleId::CPU => {
                    satisfy_opt!(self.cpu);
                    schedule_opt!(self.cpu);
                }
                ModuleId::Memory => {
                    satisfy_opt!(self.memory);
                    schedule_opt!(self.memory);
                }
                ModuleId::Timer => {
                    let tick = self.timer.satisfy(satisfy, res);
                    schedule!(self.timer);

                    self.clock.tick();

                    if let Some(weather) = &mut self.weather {
                        weather.tick(tick);
                        schedule!(weather);
                    }

                    if let Some(cpu) = &mut self.cpu {
                        cpu.tick(tick);
                        schedule!(cpu);
                    }

                    if let Some(memory) = &mut self.memory {
                        memory.tick(tick);
                        schedule!(memory);
                    }

                    self.sound.tick(tick);
                    schedule_opt!(self.session_dbus);
                }
            }

            IoUring::cqe_seen(cqe);
        }

        IoUring::submit_if_dirty();

        while let Some(event) = self.events.pop_front() {
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
                self.hyprland_queue.enqueue_dispatch(format!($($arg)*), );
                schedule_opt!(self.hyprland_writer);
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
                self.tray.trigger(uuid);
                schedule_opt!(self.session_dbus);
            }
        }

        IoUring::submit_if_dirty();
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
    let uuid = unsafe { std::ffi::CStr::from_ptr(uuid) }
        .to_str()
        .unwrap_or_else(|err| report_and_exit!("{:?}", err));

    process_command(Command::TriggerTray {
        uuid: ShortString::from(uuid),
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

fn schedule_wanted(wants: Wants, module_id: ModuleId) {
    match wants {
        Wants::Socket { domain, r#type } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_socket(domain, r#type, 0, 0);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Socket));
        }
        Wants::Connect { fd, addr, addrlen } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_connect(fd, addr, addrlen);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Connect));
        }
        Wants::Read { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_read(fd, buf, len);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Read));
        }
        Wants::Write { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_write(fd, buf, len);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Write));
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
            sqe.set_user_data(UserData::new(module_id, Satisfy::Read));

            let mut sqe = IoUring::get_sqe();
            sqe.prep_write(fd, writebuf, writelen);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Write));
        }
        Wants::OpenAt {
            dfd,
            path,
            flags,
            mode,
        } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_openat(dfd, path, flags, mode);
            sqe.set_user_data(UserData::new(module_id, Satisfy::OpenAt));
        }
        Wants::Close { fd } => {
            let mut sqe = IoUring::get_sqe();
            sqe.prep_close(fd);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Close));
        }
        Wants::Nothing => {}
    }
}
