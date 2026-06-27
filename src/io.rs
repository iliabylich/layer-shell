use crate::{
    Event,
    command::Command,
    config::{Config, IOConfig},
    event_queue::EventQueue,
    liburing::IoUring,
    modules::{
        CPU, Clock, Control, ControlRequest, KbMod, Location, Memory, Network, Niri, SessionDBus,
        Sound, SystemDBus, Timer, Tray, Weather,
    },
    sansio::{Https, Satisfy},
    user_data::{ModuleId, UserData},
};
use anyhow::{Context, Result};
use std::{assert_matches, os::fd::AsRawFd};

pub(crate) struct IO {
    ring: IoUring,
    events: EventQueue,

    config: Config,
    pub(crate) io_config: *const IOConfig,

    timer: Timer,

    session_dbus: SessionDBus,
    session_dbus_readbuf: Vec<u8>,
    sound: Sound,
    tray: Tray,

    system_dbus: SystemDBus,
    system_dbus_readbuf: Vec<u8>,
    network: Network,

    location: Location,
    weather: Weather,

    cpu: CPU,
    memory: Memory,

    kb_mod: KbMod,
    niri: Niri,

    on_event: (
        extern "C" fn(event: *const Event, *mut std::ffi::c_void),
        *mut std::ffi::c_void,
    ),
    running: bool,
}

impl IO {
    pub(crate) fn init() -> Result<()> {
        env_logger::try_init()?;
        Https::init()?;
        SessionDBus::init()?;
        SystemDBus::init()?;
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.ring.deinit();
    }

    pub(crate) fn new(
        on_event: (
            extern "C" fn(event: *const Event, *mut std::ffi::c_void),
            *mut std::ffi::c_void,
        ),
    ) -> Result<Self> {
        let config = Config::read()?;
        let io_config = IOConfig::new(&config);

        let mut this = Self {
            ring: IoUring::new(10, 0),
            events: EventQueue::new(),

            config,
            io_config,

            timer: Timer::new(),

            session_dbus: SessionDBus::new(),
            session_dbus_readbuf: vec![0; 400 * 1_024],
            sound: Sound::new(),
            tray: Tray::new(),

            system_dbus: SystemDBus::new(),
            system_dbus_readbuf: vec![0; 400 * 1_024],
            network: Network::new(),

            location: Location::new(),
            weather: Weather::new(),

            cpu: CPU::new(),
            memory: Memory::new(),

            kb_mod: KbMod::new(),
            niri: Niri::new(),

            on_event,
            running: true,
        };

        this.start()?;

        Ok(this)
    }

    fn start(&mut self) -> Result<()> {
        self.schedule_timer();

        self.schedule_location();
        self.schedule_cpu();
        self.schedule_memory();
        self.schedule_kb_mod();
        self.schedule_niri();

        self.sound.start();
        Control::init()?;
        Tray::init()?;
        self.schedule_session_dbus();

        self.network.init();
        self.schedule_system_dbus();

        self.ring.submit_if_dirty();
        Ok(())
    }

    fn on_control_req(&mut self, req: ControlRequest) {
        self.events.push_back(match req {
            ControlRequest::Exit => Event::Exit,
            ControlRequest::ToggleSessionScreen => Event::ToggleSessionScreen,
        });
    }

    pub(crate) fn handle_readable(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        while let Some(cqe) = self.ring.try_get_cqe() {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::try_from(user_data)?;
            let satisfy = Satisfy::new(op, res);
            log::trace!(target: module_id.as_str(), "Satisfy {satisfy:?}");

            match module_id {
                ModuleId::GeoLocation => self.satisfy_location(satisfy),
                ModuleId::Weather => self.satisfy_weather(satisfy),
                ModuleId::KbMod => self.satisfy_kb_mod(satisfy),
                ModuleId::Niri => self.satisfy_niri(satisfy),
                ModuleId::SessionDBus => self.satisfy_session_dbus(satisfy),
                ModuleId::SystemDBus => self.satisfy_system_dbus(satisfy),
                ModuleId::Cpu => self.satisfy_cpu(satisfy),
                ModuleId::Memory => self.satisfy_memory(satisfy),
                ModuleId::Timer => self.satisfy_timer(satisfy),
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty();

        while let Some(event) = self.events.pop_front() {
            log::info!(target: "IO", "{event:?}");
            let (callback, data) = self.on_event;
            (callback)(&raw const event, data);
        }

        Ok(())
    }

    pub(crate) fn wait_readable(&mut self) {
        self.ring.submit_and_wait(1);
    }

    pub(crate) fn process_command(&mut self, cmd: Command) {
        if !self.running {
            return;
        }

        match cmd {
            Command::Lock => spawn(&self.config.lock),
            Command::Reboot => spawn(&self.config.reboot),
            Command::Shutdown => spawn(&self.config.shutdown),
            Command::Logout => spawn(&self.config.logout),
            Command::SpawnWiFiEditor => spawn(&self.config.edit_wifi),
            Command::SpawnBluetoothEditor => spawn(&self.config.edit_bluetooth),
            Command::SpawnSystemMonitor => spawn(&self.config.open_system_monitor),
            Command::ChangeWallpaper => spawn(&self.config.change_wallpaper),

            Command::TriggerTray { uuid } => {
                self.tray.trigger(uuid.as_str());
                self.schedule_session_dbus();
            }
        }

        self.ring.submit_if_dirty();
    }
}

impl IO {
    fn schedule_timer(&mut self) {
        let Some(wants) = self.timer.wants() else {
            return;
        };
        log::trace!(target: "Timer", "{wants:?}");
        assert_matches!(self.timer.wants(), None);
        self.ring.schedule(ModuleId::Timer, wants);
    }

    fn satisfy_timer(&mut self, satisfy: Satisfy) {
        if let Some(tick) = self.timer.satisfy(satisfy, &mut self.events) {
            self.schedule_timer();

            Clock::tick(&mut self.events);

            self.weather.tick(tick);
            self.schedule_weather();

            self.cpu.tick();
            self.schedule_cpu();

            self.memory.tick();
            self.schedule_memory();

            self.sound.tick(tick);
            self.schedule_session_dbus();
        }
    }
}

impl IO {
    fn schedule_location(&mut self) {
        let Some(wants) = self.location.wants() else {
            return;
        };
        log::trace!(target: "Location", "{wants:?}");
        assert_matches!(self.location.wants(), None);
        self.ring.schedule(ModuleId::GeoLocation, wants);
    }
    fn satisfy_location(&mut self, satisfy: Satisfy) {
        if let Some((lat, lng)) = self.location.satisfy(satisfy, &mut self.events) {
            self.weather.setup(lat, lng);
            self.schedule_weather();
        } else {
            self.schedule_location();
        }
    }
}

impl IO {
    fn schedule_weather(&mut self) {
        let Some(wants) = self.weather.wants() else {
            return;
        };
        log::trace!(target: "Weather", "{wants:?}");
        assert_matches!(self.weather.wants(), None);
        self.ring.schedule(ModuleId::Weather, wants);
    }

    fn satisfy_weather(&mut self, satisfy: Satisfy) {
        self.weather.satisfy(satisfy, &mut self.events);
        self.schedule_weather();
    }
}

impl IO {
    fn schedule_cpu(&mut self) {
        let Some(wants) = self.cpu.wants() else {
            return;
        };
        log::trace!(target: "CPU", "{wants:?}");
        assert_matches!(self.cpu.wants(), None);
        self.ring.schedule(ModuleId::Cpu, wants);
    }

    fn satisfy_cpu(&mut self, satisfy: Satisfy) {
        self.cpu.satisfy(satisfy, &mut self.events);
        self.schedule_cpu();
    }
}

impl IO {
    fn schedule_memory(&mut self) {
        let Some(wants) = self.memory.wants() else {
            return;
        };
        log::trace!(target: "Memory", "{wants:?}");
        assert_matches!(self.memory.wants(), None);
        self.ring.schedule(ModuleId::Memory, wants);
    }

    fn satisfy_memory(&mut self, satisfy: Satisfy) {
        self.memory.satisfy(satisfy, &mut self.events);
        self.schedule_memory();
    }
}

impl IO {
    fn schedule_kb_mod(&mut self) {
        let Some(wants) = self.kb_mod.wants() else {
            return;
        };
        log::trace!(target: "KbMod", "{wants:?}");
        assert_matches!(self.kb_mod.wants(), None);
        self.ring.schedule(ModuleId::KbMod, wants);
    }

    fn satisfy_kb_mod(&mut self, satisfy: Satisfy) {
        self.kb_mod.satisfy(satisfy, &mut self.events);
        self.schedule_kb_mod();
    }
}

impl IO {
    fn schedule_niri(&mut self) {
        let Some(wants) = self.niri.wants() else {
            return;
        };
        log::trace!(target: "Niri", "{wants:?}");
        assert_matches!(self.niri.wants(), None);
        self.ring.schedule(ModuleId::Niri, wants);
    }

    fn satisfy_niri(&mut self, satisfy: Satisfy) {
        self.niri.satisfy(satisfy, &mut self.events);
        self.schedule_niri();
    }
}

impl IO {
    fn schedule_session_dbus(&mut self) {
        let Some(wants) = self.session_dbus.wants(&mut self.session_dbus_readbuf) else {
            return;
        };
        log::trace!(target: "SessionDBus", "{wants:?}");
        assert_matches!(
            self.session_dbus.wants(&mut self.session_dbus_readbuf),
            None
        );
        self.ring.schedule(ModuleId::SessionDBus, wants);
    }

    fn satisfy_session_dbus(&mut self, satisfy: Satisfy) {
        let message = self
            .session_dbus
            .satisfy(satisfy, &self.session_dbus_readbuf);

        if let Some(message) = message {
            self.sound.handle(message, &mut self.events);
            self.tray.handle(message, &mut self.events);

            if let Some(req) = Control::handle(message) {
                self.on_control_req(req);
            }
        }

        self.schedule_session_dbus();
    }
}

impl IO {
    fn schedule_system_dbus(&mut self) {
        let Some(wants) = self.system_dbus.wants(&mut self.system_dbus_readbuf) else {
            return;
        };
        log::trace!(target: "SystemDBus", "{wants:?}");
        assert_matches!(self.system_dbus.wants(&mut self.system_dbus_readbuf), None);
        self.ring.schedule(ModuleId::SystemDBus, wants);
    }

    fn satisfy_system_dbus(&mut self, satisfy: Satisfy) {
        let message = self.system_dbus.satisfy(satisfy, &self.system_dbus_readbuf);

        if let Some(message) = message {
            self.network.handle(message, &mut self.events);
        }

        self.schedule_system_dbus();
    }
}

impl AsRawFd for IO {
    fn as_raw_fd(&self) -> i32 {
        self.ring.as_raw_fd()
    }
}

fn spawn(cmd: &str) {
    if let Err(err) = try_spawn(cmd) {
        log::error!("{err:?}");
    }
}

fn try_spawn(cmd: &str) -> Result<()> {
    use std::process::{Command, Stdio};

    let mut cmd = cmd.split_whitespace();
    let first = cmd.next().context("command can't be parsed")?;
    let home = std::env::var("HOME").context("no $HOME")?;
    let rest = cmd.map(|arg| arg.replace('~', &home)).collect::<Vec<_>>();

    Command::new(first)
        .args(rest)
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .context("failed to spawn")
}
