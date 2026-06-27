use crate::{
    Event,
    actor::WantsSatisfy,
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
    utils::dbus::queue::{SessionDBusQueue, SystemDBusQueue},
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
    session_dbus_queue: SessionDBusQueue,
    sound: Sound,
    tray: Tray,

    system_dbus: SystemDBus,
    system_dbus_readbuf: Vec<u8>,
    system_dbus_queue: SystemDBusQueue,
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

        let mut ring = IoUring::new(10, 0);
        let events = EventQueue::new();

        let mut timer = Timer::new();
        schedule_timer(&mut timer, &mut ring);

        let mut session_dbus = SessionDBus::new();
        let mut session_dbus_readbuf = vec![0; 400 * 1_024];
        let mut session_dbus_queue = SessionDBusQueue::new()?;
        let sound = Sound::new(&mut session_dbus_queue);
        let tray = Tray::new(&mut session_dbus_queue)?;
        Control::init(&mut session_dbus_queue)?;
        schedule_session_dbus(
            &mut session_dbus,
            &mut session_dbus_readbuf,
            &session_dbus_queue,
            &mut ring,
        );

        let mut system_dbus = SystemDBus::new();
        let mut system_dbus_readbuf = vec![0; 400 * 1_024];
        let mut system_dbus_queue = SystemDBusQueue::new()?;
        let network = Network::new(&mut system_dbus_queue);
        schedule_system_dbus(
            &mut system_dbus,
            &mut system_dbus_readbuf,
            &system_dbus_queue,
            &mut ring,
        );

        let mut location = Location::new();
        schedule_location(&mut location, &mut ring);

        let weather = Weather::new();

        let mut cpu = CPU::new();
        schedule_cpu(&mut cpu, &mut ring);

        let mut memory = Memory::new();
        schedule_memory(&mut memory, &mut ring);

        let mut kb_mod = KbMod::new();
        schedule_kb_mod(&mut kb_mod, &mut ring);

        let mut niri = Niri::new();
        schedule_niri(&mut niri, &mut ring);

        ring.submit_if_dirty();

        Ok(Self {
            ring,
            events,

            config,
            io_config,

            timer,

            session_dbus,
            session_dbus_readbuf,
            session_dbus_queue,
            sound,
            tray,

            system_dbus,
            system_dbus_readbuf,
            system_dbus_queue,
            network,

            location,
            weather,

            cpu,
            memory,

            kb_mod,
            niri,

            on_event,
            running: true,
        })
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
                ModuleId::Location => self.satisfy_location(satisfy),
                ModuleId::Weather => self.satisfy_weather(satisfy),
                ModuleId::KbMod => self.satisfy_kb_mod(satisfy),
                ModuleId::Niri => self.satisfy_niri(satisfy),
                ModuleId::SessionDBus => self.satisfy_session_dbus(satisfy),
                ModuleId::SystemDBus => self.satisfy_system_dbus(satisfy),
                ModuleId::CPU => self.satisfy_cpu(satisfy),
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
                self.tray
                    .trigger(uuid.as_str(), &mut self.session_dbus_queue);
                schedule_session_dbus(
                    &mut self.session_dbus,
                    &mut self.session_dbus_readbuf,
                    &self.session_dbus_queue,
                    &mut self.ring,
                );
            }
        }

        self.ring.submit_if_dirty();
    }
}

macro_rules! generate_simple_schedule_impl {
    ($fn:ident, $module:ident) => {
        fn $fn(module: &mut $module, ring: &mut IoUring) {
            if let Some(wants) = module.wants() {
                ring.schedule(ModuleId::$module, wants);
            };
        }
    };
}

generate_simple_schedule_impl!(schedule_timer, Timer);
generate_simple_schedule_impl!(schedule_location, Location);
generate_simple_schedule_impl!(schedule_weather, Weather);
generate_simple_schedule_impl!(schedule_cpu, CPU);
generate_simple_schedule_impl!(schedule_memory, Memory);
generate_simple_schedule_impl!(schedule_kb_mod, KbMod);
generate_simple_schedule_impl!(schedule_niri, Niri);

fn schedule_session_dbus(
    module: &mut SessionDBus,
    readbuf: &mut [u8],
    queue: &SessionDBusQueue,
    ring: &mut IoUring,
) {
    let Some(wants) = module.wants(readbuf, queue) else {
        return;
    };
    log::trace!(target: "SessionDBus", "{wants:?}");
    assert_matches!(module.wants(readbuf, queue), None);
    ring.schedule(ModuleId::SessionDBus, wants);
}
fn schedule_system_dbus(
    module: &mut SystemDBus,
    readbuf: &mut [u8],
    queue: &SystemDBusQueue,
    ring: &mut IoUring,
) {
    let Some(wants) = module.wants(readbuf, queue) else {
        return;
    };
    log::trace!(target: "SystemDBus", "{wants:?}");
    assert_matches!(module.wants(readbuf, queue), None);
    ring.schedule(ModuleId::SystemDBus, wants);
}

macro_rules! generate_simple_satisfy_impl {
    ($fn:ident, $module:ident, $schedule:ident) => {
        impl IO {
            fn $fn(&mut self, satisfy: Satisfy) {
                self.$module.satisfy(satisfy, &mut self.events);
                $schedule(&mut self.$module, &mut self.ring);
            }
        }
    };
}

impl IO {
    fn satisfy_timer(&mut self, satisfy: Satisfy) {
        if let Some(tick) = self.timer.satisfy(satisfy, &mut self.events) {
            schedule_timer(&mut self.timer, &mut self.ring);

            Clock::tick(&mut self.events);

            self.weather.tick(tick);
            schedule_weather(&mut self.weather, &mut self.ring);

            self.cpu.tick();
            schedule_cpu(&mut self.cpu, &mut self.ring);

            self.memory.tick();
            schedule_memory(&mut self.memory, &mut self.ring);

            self.sound.tick(tick, &mut self.session_dbus_queue);
            schedule_session_dbus(
                &mut self.session_dbus,
                &mut self.session_dbus_readbuf,
                &self.session_dbus_queue,
                &mut self.ring,
            );
        }
    }
}

impl IO {
    fn satisfy_location(&mut self, satisfy: Satisfy) {
        if let Some((lat, lng)) = self.location.satisfy(satisfy, &mut self.events) {
            self.weather.start(lat, lng);
            schedule_weather(&mut self.weather, &mut self.ring);
        } else {
            schedule_location(&mut self.location, &mut self.ring);
        }
    }
}

generate_simple_satisfy_impl!(satisfy_weather, weather, schedule_weather);
generate_simple_satisfy_impl!(satisfy_cpu, cpu, schedule_cpu);
generate_simple_satisfy_impl!(satisfy_memory, memory, schedule_memory);
generate_simple_satisfy_impl!(satisfy_kb_mod, kb_mod, schedule_kb_mod);
generate_simple_satisfy_impl!(satisfy_niri, niri, schedule_niri);

impl IO {
    fn satisfy_session_dbus(&mut self, satisfy: Satisfy) {
        let message = self.session_dbus.satisfy(
            satisfy,
            &self.session_dbus_readbuf,
            &mut self.session_dbus_queue,
        );

        if let Some(message) = message {
            self.sound
                .handle(message, &mut self.events, &mut self.session_dbus_queue);
            self.tray
                .handle(message, &mut self.events, &mut self.session_dbus_queue);

            if let Some(req) = Control::handle(message, &mut self.session_dbus_queue) {
                self.on_control_req(req);
            }
        }

        schedule_session_dbus(
            &mut self.session_dbus,
            &mut self.session_dbus_readbuf,
            &self.session_dbus_queue,
            &mut self.ring,
        );
    }
}

impl IO {
    fn satisfy_system_dbus(&mut self, satisfy: Satisfy) {
        let message = self.system_dbus.satisfy(
            satisfy,
            &self.system_dbus_readbuf,
            &mut self.system_dbus_queue,
        );

        if let Some(message) = message {
            self.network
                .handle(message, &mut self.events, &mut self.system_dbus_queue);
        }

        schedule_system_dbus(
            &mut self.system_dbus,
            &mut self.system_dbus_readbuf,
            &self.system_dbus_queue,
            &mut self.ring,
        );
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
