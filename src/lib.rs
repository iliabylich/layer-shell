mod command;
mod config;
mod dbus;
mod event;
mod ffi;
mod https;
mod liburing;
mod macros;
mod modules;
mod timerfd;
mod user_data;

use anyhow::Result;
use command::Command;
use config::{Config, IOConfig};
pub use event::Event;
pub use ffi::{FFIArray, FFIString};
use std::ffi::c_void;

use crate::{
    dbus::{DBus, messages::org_freedesktop_dbus::Hello},
    liburing::IoUring,
    macros::report_and_exit,
    modules::{
        CPU, Clock, Control, ControlRequest, Hyprland, Location, Memory, Network, Sound, Tray,
        Weather,
    },
    timerfd::Timerfd,
    user_data::{ModuleId, UserData},
};

struct IO {
    config: Config,
    io_config: *const IOConfig,

    timer: Box<Timerfd>,
    session_dbus: Box<DBus>,
    system_dbus: Box<DBus>,

    location: Box<Location>,
    weather: Box<Weather>,
    hyprland: Box<Hyprland>,
    cpu: Box<CPU>,
    memory: Box<Memory>,
    sound: Box<Sound>,
    control: Box<Control>,
    network: Box<Network>,
    tray: Box<Tray>,

    on_event: extern "C" fn(event: *const Event),
    running: bool,
}

impl IO {
    fn try_new(on_event: extern "C" fn(event: *const Event)) -> Result<Self> {
        let config = Config::read()?;
        let io_config = Box::leak(Box::new(IOConfig::from(&config)));

        let mut this = Self {
            config,
            io_config,

            timer: Timerfd::new(),
            session_dbus: DBus::new_session()?,
            system_dbus: DBus::new_system()?,

            location: Location::new(),
            weather: Weather::new(),
            hyprland: Hyprland::new(),
            cpu: CPU::new(),
            memory: Memory::new(),
            sound: Sound::new(),
            control: Control::new(),
            network: Network::new(),
            tray: Tray::new(),

            on_event,
            running: true,
        };

        this.init();

        Ok(this)
    }

    fn init(&mut self) {
        self.timer.init();

        self.location.init();
        self.hyprland.init();
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

    fn new(on_event: extern "C" fn(event: *const Event)) -> Self {
        Self::try_new(on_event).unwrap_or_else(|err| report_and_exit!("{err:?}"))
    }

    fn from_raw(ptr: *mut c_void) -> &'static mut Self {
        unsafe { ptr.cast::<Self>().as_mut() }
            .unwrap_or_else(|| report_and_exit!("NULL IO pointer given to IO::from_raw()"))
    }

    fn on_control_req(&mut self, req: ControlRequest, events: &mut Vec<Event>) {
        match req {
            ControlRequest::CapsLockToggled => {
                self.hyprland.enqueue_get_caps_lock();
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
                    if let Some((lat, lng)) = self.location.process(op, res) {
                        self.weather.init(lat, lng);
                    }
                }
                ModuleId::Weather => {
                    self.weather.process(op, res, &mut events);
                }
                ModuleId::HyprlandReader => {
                    self.hyprland.process_reader(op, res, &mut events);
                }
                ModuleId::HyprlandWriter => {
                    self.hyprland.process_writer(op, res, &mut events);
                }

                ModuleId::SessionDBusAuth => {
                    self.session_dbus.process_auth(op, res);
                }
                ModuleId::SessionDBusReader => {
                    if let Some(message) = self.session_dbus.process_read(op, res) {
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
                ModuleId::SessionDBusWriter => {
                    self.session_dbus.process_write(op, res);
                }

                ModuleId::SystemDBusAuth => {
                    self.system_dbus.process_auth(op, res);
                }
                ModuleId::SystemDBusReader => {
                    if let Some(message) = self.system_dbus.process_read(op, res) {
                        self.network
                            .on_message(&mut self.system_dbus, &message, &mut events);
                    }
                }
                ModuleId::SystemDBusWriter => {
                    self.system_dbus.process_write(op, res);
                }

                ModuleId::CPU => {
                    self.cpu.process(op, res, &mut events);
                }
                ModuleId::Memory => {
                    self.memory.process(op, res, &mut events);
                }
                ModuleId::TimerFD => {
                    let tick = self.timer.process(op);
                    Clock::tick(&mut events);
                    self.weather.tick(tick);
                    self.cpu.tick();
                    self.memory.tick();
                }
            }

            IoUring::cqe_seen(cqe);
        }

        IoUring::submit_if_dirty();

        for event in events {
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
                self.hyprland.dispatch(format!($($arg)*), );
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

#[unsafe(no_mangle)]
pub extern "C" fn io_init(on_event: extern "C" fn(event: *const Event)) -> *mut c_void {
    pretty_env_logger::init();
    IoUring::init(10, 0);
    (Box::leak(Box::new(IO::new(on_event))) as *mut IO).cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit(io: *mut c_void) {
    IO::from_raw(io).deinit();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable(io: *mut c_void) {
    IO::from_raw(io).handle_readable();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable(io: *mut c_void) {
    IO::from_raw(io).wait_readable();
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    IoUring::as_raw_fd()
}

#[unsafe(no_mangle)]
pub extern "C" fn io_get_config(io: *mut c_void) -> *const IOConfig {
    IO::from_raw(io).io_config
}

fn process_command(io: *mut c_void, cmd: Command) {
    IO::from_raw(io).process_command(cmd);
}

#[unsafe(no_mangle)]
pub extern "C" fn io_hyprland_go_to_workspace(io: *mut c_void, workspace: usize) {
    process_command(io, Command::GoToWorkspace { workspace });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_lock(io: *mut c_void) {
    process_command(io, Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot(io: *mut c_void) {
    process_command(io, Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown(io: *mut c_void) {
    process_command(io, Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout(io: *mut c_void) {
    process_command(io, Command::Logout);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(io: *mut c_void, uuid: *const std::ffi::c_char) {
    process_command(
        io,
        Command::TriggerTray {
            uuid: FFIString::from(uuid).into(),
        },
    );
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor(io: *mut c_void) {
    process_command(io, Command::SpawnWiFiEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor(io: *mut c_void) {
    process_command(io, Command::SpawnBluetoothEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor(io: *mut c_void) {
    process_command(io, Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme(io: *mut c_void) {
    process_command(io, Command::ChangeTheme);
}
