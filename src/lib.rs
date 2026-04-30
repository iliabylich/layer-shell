#![expect(static_mut_refs)]

mod command;
mod config;
mod event;
mod event_queue;
mod ffi;
mod liburing;
mod modules;
mod sansio;
mod timer;
mod unix_socket;
mod user_data;
mod utils;

use command::Command;
use config::{Config, IOConfig};
pub use event::Event;
pub use ffi::FFIArray;

use crate::{
    event_queue::EventQueue,
    liburing::IoUring,
    modules::{
        CPU, CapsLock, Clock, Control, ControlRequest, Location, Memory, Network, Niri,
        SessionDBus, Sound, SystemDBus, Tray, Weather,
    },
    sansio::{Satisfy, Wants},
    timer::Timer,
    user_data::{ModuleId, UserData},
    utils::{Logger, StringRef},
};
use anyhow::{Context, Result, bail};

struct IO {
    config: Config,
    io_config: *const IOConfig,

    timer: Timer,
    clock: Clock,

    session_dbus: SessionDBus,
    sound: Sound,
    control: Control,
    tray: Tray,
    system_dbus: SystemDBus,
    network: Network,

    location: Location,
    weather: Weather,
    cpu: CPU,
    memory: Memory,
    caps_lock: CapsLock,
    niri: Niri,

    on_event: extern "C" fn(event: *const Event),
    running: bool,
    logging_enabled: bool,
}

static mut GLOBAL_IO: *mut IO = core::ptr::null_mut();

macro_rules! schedule {
    ($module:expr) => {{
        if let Some(wants) = $module.wants()? {
            let module_id = $module.module_id();
            if let Some(wants_next) = $module.wants()? {
                anyhow::bail!("Module {module_id:?} wants {wants_next:?} after {wants:?}");
            }
            schedule_wanted(wants, module_id)?;
        }
    }};
}

impl IO {
    fn new(on_event: extern "C" fn(event: *const Event), logging_enabled: bool) -> Result<Self> {
        let config = Config::read()?;
        let io_config = Box::leak(Box::new(IOConfig::try_from(&config)?));

        let mut this = Self {
            config,
            io_config,

            timer: Timer::new()?,
            clock: Clock::new(),

            session_dbus: SessionDBus::new(),
            sound: Sound::new(),
            control: Control::new(),
            tray: Tray::new(),
            system_dbus: SystemDBus::new(),
            network: Network::new(),

            location: Location::new(),
            weather: Weather::new(),
            cpu: CPU::new(),
            memory: Memory::new(),
            caps_lock: CapsLock::new(),
            niri: Niri::new()?,

            on_event,
            running: true,
            logging_enabled,
        };

        this.init()?;

        Ok(this)
    }

    fn init(&mut self) -> Result<()> {
        schedule!(self.timer);

        schedule!(self.location);
        schedule!(self.cpu);
        schedule!(self.memory);
        schedule!(self.caps_lock);
        schedule!(self.niri);

        self.sound.init()?;
        self.control.init();
        self.tray.init()?;
        schedule!(self.session_dbus);

        self.network.init()?;
        schedule!(self.system_dbus);

        IoUring::submit_if_dirty()?;
        Ok(())
    }

    fn on_control_req(&mut self, req: ControlRequest) {
        match req {
            ControlRequest::Exit => EventQueue::push_back(Event::Exit),
            ControlRequest::ReloadStyles => EventQueue::push_back(Event::ReloadStyles),
            ControlRequest::ToggleSessionScreen => {
                EventQueue::push_back(Event::ToggleSessionScreen)
            }
        }
    }

    fn handle_readable(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        while let Some(cqe) = IoUring::try_get_cqe()? {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::try_from(user_data)?;
            let satisfy = Satisfy::try_from(op)?;

            macro_rules! satisfy {
                ($module:expr) => {
                    $module.satisfy(satisfy, res)
                };
            }

            match module_id {
                ModuleId::GeoLocation => {
                    if let Some((lat, lng)) = satisfy!(self.location) {
                        self.weather.setup(lat, lng);
                        schedule!(self.weather);
                    } else {
                        schedule!(self.location);
                    }
                }

                ModuleId::Weather => {
                    satisfy!(self.weather);
                    schedule!(self.weather);
                }

                ModuleId::CapsLock => {
                    satisfy!(self.caps_lock);
                    schedule!(self.caps_lock);
                }

                ModuleId::Niri => {
                    satisfy!(self.niri);
                    schedule!(self.niri);
                }

                ModuleId::SessionDBus => {
                    let message = satisfy!(self.session_dbus);

                    if let Some(message) = message {
                        self.sound.on_message(message);
                        self.tray.on_message(message)?;

                        if let Some(req) = self.control.on_message(message) {
                            self.on_control_req(req);
                        }
                    }

                    schedule!(self.session_dbus);
                }

                ModuleId::SystemDBus => {
                    let message = satisfy!(self.system_dbus);

                    if let Some(message) = message {
                        self.network.on_message(message)?;
                    }

                    schedule!(self.system_dbus);
                }

                ModuleId::CPU => {
                    satisfy!(self.cpu);
                    schedule!(self.cpu);
                }
                ModuleId::Memory => {
                    satisfy!(self.memory);
                    schedule!(self.memory);
                }
                ModuleId::Timer => {
                    let tick = self.timer.satisfy(satisfy, res)?;
                    schedule!(self.timer);

                    self.clock.tick();

                    self.weather.tick(tick);
                    schedule!(self.weather);

                    self.cpu.tick(tick);
                    schedule!(self.cpu);

                    self.memory.tick(tick);
                    schedule!(self.memory);

                    self.sound.tick(tick)?;
                    schedule!(self.session_dbus);
                }
            }

            IoUring::cqe_seen(cqe);
        }

        IoUring::submit_if_dirty()?;

        while let Some(event) = EventQueue::pop_front() {
            if self.logging_enabled {
                log::info!(target: "IO", "{event:?}");
            }
            (self.on_event)(&event);
        }

        Ok(())
    }

    fn wait_readable(&mut self) -> Result<()> {
        IoUring::submit_and_wait(1)
    }

    fn process_command(&mut self, cmd: Command) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        match cmd {
            Command::Lock => {
                spawn(&self.config.lock)?;
            }
            Command::Reboot => {
                spawn(&self.config.reboot)?;
            }
            Command::Shutdown => {
                spawn(&self.config.shutdown)?;
            }
            Command::Logout => {
                spawn(&self.config.logout)?;
            }
            Command::SpawnWiFiEditor => {
                spawn(&self.config.edit_wifi)?;
            }
            Command::SpawnBluetoothEditor => {
                spawn(&self.config.edit_bluetooth)?;
            }
            Command::SpawnSystemMonitor => {
                spawn(&self.config.open_system_monitor)?;
            }
            Command::ChangeTheme => {
                spawn(&self.config.change_theme)?;
            }

            Command::TriggerTray { uuid } => {
                self.tray.trigger(uuid)?;
                schedule!(self.session_dbus);
            }
        }

        IoUring::submit_if_dirty()?;
        Ok(())
    }

    fn deinit(&mut self) {
        self.running = false;
        IoUring::deinit();
    }

    fn global() -> Result<&'static mut IO> {
        unsafe { GLOBAL_IO.as_mut() }.context("IO is not initialized. Call io_init() first.")
    }
}

macro_rules! try_or {
    ($fallback:expr; $code:expr) => {{
        match std::panic::catch_unwind(|| $code) {
            Ok(Ok(out)) => out,
            Ok(Err(err)) => {
                let err: anyhow::Error = err;
                log::error!("error returned: {err:?}");
                $fallback
            }
            Err(err) => {
                log::error!("panic: {err:?}");
                $fallback
            }
        }
    }};
}
macro_rules! try_or_null {
    ($code:block) => {
        try_or!(std::ptr::null_mut(); $code)
    };
}
macro_rules! try_or_false {
    ($code:block) => {
        try_or!(false; $code)
    };
}

#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn io_init(
    on_event: extern "C" fn(event: *const Event),
    logging_enabled: bool,
) -> bool {
    try_or_false!({
        if unsafe { !GLOBAL_IO.is_null() } {
            bail!("io_init() called while IO is already initialized");
        }

        Logger::init()?;

        rustls_openssl::default_provider()
            .install_default()
            .map_err(|_err| anyhow::anyhow!("failed to install OpenSSL CryptoProvider"))?;
        IoUring::init(10, 0)?;
        unsafe {
            GLOBAL_IO = Box::into_raw(Box::new(IO::new(on_event, logging_enabled)?));
        }
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_deinit() -> bool {
    try_or_false!({
        if unsafe { GLOBAL_IO.is_null() } {
            bail!("io_deinit() called while IO is not initialized");
        }

        unsafe {
            (*GLOBAL_IO).deinit();
            drop(Box::from_raw(GLOBAL_IO));
            GLOBAL_IO = core::ptr::null_mut();
        }
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_handle_readable() -> bool {
    try_or_false!({
        IO::global()?.handle_readable()?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_wait_readable() -> bool {
    try_or_false!({
        IO::global()?.wait_readable()?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_as_raw_fd() -> i32 {
    IoUring::as_raw_fd()
}

#[unsafe(no_mangle)]
pub extern "C" fn io_get_config() -> *const IOConfig {
    try_or_null!({ Ok(IO::global()?.io_config) })
}

#[unsafe(no_mangle)]
pub extern "C" fn io_lock() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::Lock)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::Reboot)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::Shutdown)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::Logout)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn io_trigger_tray(uuid: *const core::ffi::c_char) -> bool {
    try_or_false!({
        let uuid = unsafe { std::ffi::CStr::from_ptr(uuid) }.to_str()?;

        IO::global()?.process_command(Command::TriggerTray {
            uuid: StringRef::new(uuid)?,
        })?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::SpawnWiFiEditor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::SpawnBluetoothEditor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::SpawnSystemMonitor)?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme() -> bool {
    try_or_false!({
        IO::global()?.process_command(Command::ChangeTheme)?;
        Ok(true)
    })
}

fn schedule_wanted(wants: Wants, module_id: ModuleId) -> Result<()> {
    match wants {
        Wants::Socket { domain, r#type } => {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_socket(domain, r#type, 0, 0);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Socket));
        }
        Wants::Connect { fd, addr, addrlen } => {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_connect(fd, addr, addrlen);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Connect));
        }
        Wants::Read { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_read(fd, buf, len);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Read));
        }
        Wants::Write { fd, buf, len } => {
            let mut sqe = IoUring::get_sqe()?;
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
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_read(fd, readbuf, readlen);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Read));

            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_write(fd, writebuf, writelen);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Write));
        }
        Wants::OpenAt {
            dfd,
            path,
            flags,
            mode,
        } => {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_openat(dfd, path, flags, mode);
            sqe.set_user_data(UserData::new(module_id, Satisfy::OpenAt));
        }
        Wants::Close { fd } => {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_close(fd);
            sqe.set_user_data(UserData::new(module_id, Satisfy::Close));
        }
    }

    Ok(())
}

fn spawn(cmd: &str) -> Result<()> {
    use std::process::{Command, Stdio};

    let mut cmd = cmd.split_whitespace();
    let first = cmd.next().context("command can't be pased")?;
    let rest = cmd.collect::<Vec<_>>();

    Command::new(first)
        .args(rest)
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .context("failed to spawn")
}
