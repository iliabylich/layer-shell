mod command;
mod config;
mod dbus;
mod event;
mod ffi;
mod https;
mod liburing;
mod modules;
mod timerfd;
mod user_data;

use anyhow::Result;
use command::Command;
use config::{Config, IOConfig};
pub use event::Event;
pub use ffi::{CArray, CString};
use std::{ffi::c_void, os::fd::AsRawFd};

use crate::{
    dbus::{DBus, messages::org_freedesktop_dbus::Hello},
    liburing::IoUring,
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

    ring: IoUring,

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
}

impl IO {
    fn try_new(on_event: extern "C" fn(event: *const Event)) -> Result<Self> {
        let config = Config::read()?;
        let io_config = Box::leak(Box::new(IOConfig::from(&config)));

        let mut this = Self {
            config,
            io_config,

            ring: IoUring::new(10, 0)?,

            timer: Timerfd::new()?,
            session_dbus: DBus::new_session()?,
            system_dbus: DBus::new_system()?,

            location: Location::new()?,
            weather: Weather::new()?,
            hyprland: Hyprland::new()?,
            cpu: CPU::new()?,
            memory: Memory::new()?,
            sound: Sound::new(),
            control: Control::new(),
            network: Network::new(),
            tray: Tray::new(),
            on_event,
        };

        this.init()?;

        Ok(this)
    }

    fn init(&mut self) -> Result<()> {
        self.timer.init(&mut self.ring)?;

        self.location.init(&mut self.ring)?;
        self.hyprland.init(&mut self.ring)?;

        self.session_dbus
            .enqueue(&mut Hello.into(), &mut self.ring)?;
        self.sound.init(&mut self.session_dbus, &mut self.ring)?;
        self.control.init(&mut self.session_dbus, &mut self.ring)?;
        self.tray.init(&mut self.session_dbus, &mut self.ring)?;
        self.session_dbus.init(&mut self.ring)?;

        self.system_dbus
            .enqueue(&mut Hello.into(), &mut self.ring)?;
        self.network.init(&mut self.system_dbus, &mut self.ring)?;
        self.system_dbus.init(&mut self.ring)?;

        self.ring.submit()?;

        Ok(())
    }

    fn new(on_event: extern "C" fn(event: *const Event)) -> Self {
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

    fn on_control_req(&mut self, req: ControlRequest, events: &mut Vec<Event>) -> Result<()> {
        match req {
            ControlRequest::CapsLockToggled => {
                self.hyprland.enqueue_get_caps_lock(&mut self.ring)?;
                if self.ring.take_dirty() {
                    self.ring.submit()?
                }
            }
            ControlRequest::Exit => events.push(Event::Exit),
            ControlRequest::ReloadStyles => events.push(Event::ReloadStyles),
            ControlRequest::ToggleSessionScreen => events.push(Event::ToggleSessionScreen),
        }

        Ok(())
    }

    fn try_handle_readable(&mut self) -> Result<()> {
        let mut events = vec![];

        while let Some(cqe) = self.ring.try_get_cqe()? {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let Ok(UserData { module_id, op, .. }) = UserData::try_from(user_data) else {
                eprintln!("Unknown user data: {:?}", cqe.user_data());
                continue;
            };

            match module_id {
                ModuleId::GeoLocation => {
                    if let Some((lat, lng)) = self.location.process(op, res, &mut self.ring)? {
                        self.weather.init(lat, lng, &mut self.ring)?;
                    }
                }
                ModuleId::Weather => {
                    self.weather.process(op, res, &mut self.ring, &mut events)?;
                }
                ModuleId::HyprlandReader => {
                    self.hyprland
                        .process_reader(op, res, &mut self.ring, &mut events)?;
                }
                ModuleId::HyprlandWriter => {
                    self.hyprland
                        .process_writer(op, res, &mut self.ring, &mut events)?;
                }

                ModuleId::SessionDBusAuth => {
                    self.session_dbus.process_auth(op, res, &mut self.ring)?;
                }
                ModuleId::SessionDBusReader => {
                    if let Some(message) =
                        self.session_dbus.process_read(op, res, &mut self.ring)?
                    {
                        self.sound.on_message(
                            &mut self.session_dbus,
                            &message,
                            &mut events,
                            &mut self.ring,
                        )?;
                        self.tray.on_message(
                            &mut self.session_dbus,
                            &message,
                            &mut events,
                            &mut self.ring,
                        )?;

                        if let Some(req) = self.control.on_message(
                            &message,
                            &mut self.session_dbus,
                            &mut self.ring,
                        )? {
                            self.on_control_req(req, &mut events)?;
                        }
                    }
                }
                ModuleId::SessionDBusWriter => {
                    self.session_dbus.process_write(op, res, &mut self.ring)?;
                }

                ModuleId::SystemDBusAuth => {
                    self.system_dbus.process_auth(op, res, &mut self.ring)?;
                }
                ModuleId::SystemDBusReader => {
                    if let Some(message) = self.system_dbus.process_read(op, res, &mut self.ring)? {
                        self.network.on_message(
                            &mut self.system_dbus,
                            &message,
                            &mut events,
                            &mut self.ring,
                        )?;
                    }
                }
                ModuleId::SystemDBusWriter => {
                    self.system_dbus.process_write(op, res, &mut self.ring)?;
                }

                ModuleId::CPU => {
                    self.cpu.process(op, res, &mut events)?;
                }
                ModuleId::Memory => {
                    self.memory.process(op, res, &mut events)?;
                }
                ModuleId::TimerFD => {
                    let tick = self.timer.process(op, &mut self.ring)?;
                    Clock::tick(tick, &mut events);
                    self.weather.tick(tick, &mut self.ring)?;
                    self.cpu.tick(tick, &mut self.ring)?;
                    self.memory.tick(tick, &mut self.ring)?;
                }
                ModuleId::Max => unreachable!(),
            }

            self.ring.cqe_seen(cqe);
        }

        for event in events {
            (self.on_event)(&event);
        }

        if self.ring.take_dirty() {
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

    fn try_process_command(&mut self, cmd: Command) -> Result<()> {
        macro_rules! hyprctl {
            ($($arg:tt)*) => {{
                self.hyprland.dispatch(format!($($arg)*), &mut self.ring)?;

                if self.ring.take_dirty() {
                    self.ring.submit()?
                }
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

            Command::TriggerTray { uuid } => todo!("{uuid}"),
        }

        Ok(())
    }

    fn process_command(&mut self, cmd: Command) {
        self.try_process_command(cmd).unwrap_or_else(|err| {
            eprintln!("{err:?}");
            std::process::exit(1);
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init(on_event: extern "C" fn(event: *const Event)) -> *mut c_void {
    env_logger::init();
    (Box::leak(Box::new(IO::new(on_event))) as *mut IO).cast()
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
pub extern "C" fn io_as_raw_fd(io: *mut c_void) -> i32 {
    IO::from_raw(io).ring.as_raw_fd()
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
            uuid: CString::from(uuid).into(),
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
