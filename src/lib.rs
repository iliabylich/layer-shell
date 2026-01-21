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
        CPU, Clock, Control, ControlRequest, Hyprland, Memory, Network, Sound, Tray, Weather,
    },
    timerfd::Timerfd,
    user_data::UserData,
};

struct IO {
    config: Config,
    io_config: *const IOConfig,

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
        let drained = self.timer.drain(&mut self.ring)?;
        assert!(drained);

        self.weather.drain(&mut self.ring)?;
        self.hyprland.drain(&mut self.ring)?;
        self.cpu.drain(&mut self.ring)?;
        self.memory.drain(&mut self.ring)?;

        self.session_dbus.enqueue(&mut Hello.into());
        self.sound.init(&mut self.session_dbus);
        self.control.init(&mut self.session_dbus);
        self.tray.init(&mut self.session_dbus);
        self.session_dbus.drain(&mut self.ring)?;

        self.system_dbus.enqueue(&mut Hello.into());
        self.network.init(&mut self.system_dbus);
        self.system_dbus.drain(&mut self.ring)?;

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
            (self.on_event)(&event);
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

    fn on_control_req(&mut self, req: ControlRequest, events: &mut Vec<Event>) {
        match req {
            ControlRequest::CapsLockToggled => self.hyprland.enqueue_get_caps_lock(),
            ControlRequest::Exit => events.push(Event::Exit),
            ControlRequest::ReloadStyles => events.push(Event::ReloadStyles),
            ControlRequest::ToggleSessionScreen => events.push(Event::ToggleSessionScreen),
        }
    }

    fn try_handle_session_dbus(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        if let Some(message) = self.session_dbus.feed(user_data, res)? {
            self.sound.on_message(&message, events);
            self.tray
                .on_message(&mut self.session_dbus, &message, events);

            if let Some(req) = self.control.on_message(&message, &mut self.session_dbus) {
                self.on_control_req(req, events);
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
            self.network
                .on_message(&mut self.system_dbus, &message, events);
        }
        self.system_dbus.drain(&mut self.ring)
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
            (self.on_event)(&event);
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

    fn process_command(&mut self, cmd: Command) {
        macro_rules! hyprctl {
            ($($arg:tt)*) => {
                self.hyprland.dispatch(format!($($arg)*))
            };
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
