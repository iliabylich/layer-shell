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
        CPU, Clock, Control, ControlRequest, DBusQueued as _, Hyprland, HyprlandReader,
        HyprlandWriter, Location, Memory, Network, SessionDBus, Sound, SystemDBus, Tray, Weather,
    },
    timer::Timer,
    user_data::{ModuleId, UserData},
};

struct IO {
    config: Config,
    io_config: *const IOConfig,

    timer: Box<Timer>,
    session_dbus: SessionDBus,
    system_dbus: SystemDBus,

    hyprland_reader: Option<HyprlandReader>,
    hyprland_writer: Option<HyprlandWriter>,

    location: Box<Location>,
    weather: Option<Box<Weather>>,
    cpu: Box<CPU>,
    memory: Box<Memory>,
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

        let (hyprland_reader, hyprland_writer) = Hyprland::new();

        let mut this = Self {
            config,
            io_config,

            timer: Timer::new(),
            session_dbus: SessionDBus::new(),
            system_dbus: SystemDBus::new(),

            location: Location::new(),
            weather: None,
            hyprland_reader,
            hyprland_writer,
            cpu: CPU::new(),
            memory: Memory::new(),
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
        self.timer.init();

        self.location.init();
        if let Some(hyprland_reader) = &mut self.hyprland_reader {
            hyprland_reader.init();
        }
        if let Some(hyprland_writer) = &mut self.hyprland_writer {
            hyprland_writer.init();
        }
        self.cpu.init();
        self.memory.init();

        self.session_dbus.enqueue(&mut Hello.into());
        self.sound.init(&mut self.session_dbus);
        self.control.init(&mut self.session_dbus);
        self.tray.init(&mut self.session_dbus);
        self.session_dbus.init();

        self.system_dbus.enqueue(&mut Hello.into());
        self.network.init(&mut self.system_dbus);
        self.system_dbus.init();

        IoUring::submit_if_dirty();
    }

    fn on_control_req(&mut self, req: ControlRequest, events: &mut Vec<Event>) {
        match req {
            ControlRequest::CapsLockToggled => {
                if let Some(hyprland_writer) = &mut self.hyprland_writer {
                    hyprland_writer.enqueue_get_caps_lock();
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

            match module_id {
                ModuleId::GeoLocation => {
                    if let Some((lat, lng)) = self.location.satisfy(op, res) {
                        self.weather = Some(Weather::new(lat, lng));
                    }
                }

                ModuleId::Weather => {
                    if let Some(weather) = self.weather.as_deref_mut() {
                        weather.process(op, res, &mut events);
                    }
                }

                ModuleId::HyprlandReader => {
                    if let Some(hyprland_reader) = &mut self.hyprland_reader {
                        hyprland_reader.process(op, res, &mut events);
                    }
                }
                ModuleId::HyprlandWriter => {
                    if let Some(hyprland_writer) = &mut self.hyprland_writer {
                        hyprland_writer.process(op, res, &mut events);
                    }
                }

                ModuleId::SessionDBus => {
                    if let Some(message) = self.session_dbus.process(op, res) {
                        self.sound
                            .on_message(&mut self.session_dbus, &message, &mut events);
                        self.tray
                            .on_message(&mut self.session_dbus, &message, &mut events);

                        if let Some(req) = self.control.on_message(&message, &mut self.session_dbus)
                        {
                            self.on_control_req(req, &mut events);
                        }
                    }
                }

                ModuleId::SystemDBus => {
                    if let Some(message) = self.system_dbus.process(op, res) {
                        self.network
                            .on_message(&mut self.system_dbus, &message, &mut events);
                    }
                }

                ModuleId::CPU => {
                    self.cpu.process(op, res, &mut events);
                }
                ModuleId::Memory => {
                    self.memory.process(op, res, &mut events);
                }
                ModuleId::TimerFD => {
                    let tick = self.timer.process(op, res);
                    Clock::tick(&mut events);
                    if let Some(weather) = self.weather.as_deref_mut() {
                        weather.tick(tick);
                    }
                    self.cpu.tick();
                    self.memory.tick();
                    self.sound.tick(tick, &mut self.session_dbus);
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
                self.tray.trigger(&uuid, &mut self.session_dbus);
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
