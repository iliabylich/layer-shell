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
use std::os::fd::AsRawFd;

pub(crate) struct IO {
    ring: IoUring,

    config: Config,
    pub(crate) io_config: *const IOConfig,

    timer: Timer,

    session_dbus: SessionDBus,
    sound: Sound,
    tray: Tray,
    system_dbus: SystemDBus,
    network: Network,

    location: Location,
    coordinates: Option<(f64, f64)>,
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

macro_rules! schedule {
    ($module_id:ident, $module:expr, $io_uring:expr) => {{
        let module_id = ModuleId::$module_id;

        if let Some(wants) = $module.wants() {
            if let Some(wants_next) = $module.wants() {
                log::error!("Module {module_id:?} wants {wants_next:?} after {wants:?}");
                std::process::exit(1);
            }
            log::trace!(target: module_id.as_str(), "Wants {wants:?}");
            $io_uring.schedule(module_id, wants);
        }
    }};
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

            config,
            io_config,

            timer: Timer::new(),

            session_dbus: SessionDBus::new(),
            sound: Sound::new(),
            tray: Tray::new(),
            system_dbus: SystemDBus::new(),
            network: Network::new(),

            location: Location::new(),
            coordinates: None,
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
        schedule!(Timer, self.timer, &mut self.ring);

        schedule!(GeoLocation, self.location, &mut self.ring);
        schedule!(CPU, self.cpu, &mut self.ring);
        schedule!(Memory, self.memory, &mut self.ring);
        schedule!(KbMod, self.kb_mod, &mut self.ring);
        schedule!(Niri, self.niri, &mut self.ring);

        self.sound.start();
        Control::init()?;
        Tray::init()?;
        schedule!(SessionDBus, self.session_dbus, &mut self.ring);

        self.network.init();
        schedule!(SystemDBus, self.system_dbus, &mut self.ring);

        self.ring.submit_if_dirty();
        Ok(())
    }

    fn on_control_req(req: ControlRequest) {
        EventQueue::push_back(match req {
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

            macro_rules! satisfy {
                ($module:expr) => {
                    $module.satisfy(satisfy)
                };
            }

            match module_id {
                ModuleId::GeoLocation => {
                    if let Some((lat, lng)) = satisfy!(self.location) {
                        self.coordinates = Some((lat, lng));
                        self.weather.setup(lat, lng);
                        schedule!(Weather, self.weather, &mut self.ring);
                    } else {
                        schedule!(GeoLocation, self.location, &mut self.ring);
                    }
                }

                ModuleId::Weather => {
                    satisfy!(self.weather);
                    schedule!(Weather, self.weather, &mut self.ring);
                }

                ModuleId::KbMod => {
                    satisfy!(self.kb_mod);
                    schedule!(KbMod, self.kb_mod, &mut self.ring);
                }

                ModuleId::Niri => {
                    satisfy!(self.niri);
                    schedule!(Niri, self.niri, &mut self.ring);
                }

                ModuleId::SessionDBus => {
                    let message = satisfy!(self.session_dbus);

                    if let Some(message) = message {
                        self.sound.handle(message);
                        self.tray.handle(message);

                        if let Some(req) = Control::handle(message) {
                            Self::on_control_req(req);
                        }
                    }

                    schedule!(SessionDBus, self.session_dbus, &mut self.ring);
                }

                ModuleId::SystemDBus => {
                    let message = satisfy!(self.system_dbus);

                    if let Some(message) = message {
                        self.network.handle(message);
                    }

                    schedule!(SystemDBus, self.system_dbus, &mut self.ring);
                }

                ModuleId::CPU => {
                    satisfy!(self.cpu);
                    schedule!(CPU, self.cpu, &mut self.ring);
                }
                ModuleId::Memory => {
                    satisfy!(self.memory);
                    schedule!(Memory, self.memory, &mut self.ring);
                }
                ModuleId::Timer => {
                    if let Some(tick) = satisfy!(self.timer) {
                        schedule!(Timer, self.timer, &mut self.ring);

                        Clock::tick();

                        self.weather.tick(tick);
                        schedule!(Weather, self.weather, &mut self.ring);

                        self.cpu.tick();
                        schedule!(CPU, self.cpu, &mut self.ring);

                        self.memory.tick();
                        schedule!(Memory, self.memory, &mut self.ring);

                        self.sound.tick(tick);
                        schedule!(SessionDBus, self.session_dbus, &mut self.ring);
                    }
                }
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty();

        while let Some(event) = EventQueue::pop_front() {
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
            Command::Lock => {
                spawn(&self.config.lock);
            }
            Command::Reboot => {
                spawn(&self.config.reboot);
            }
            Command::Shutdown => {
                spawn(&self.config.shutdown);
            }
            Command::Logout => {
                spawn(&self.config.logout);
            }
            Command::SpawnWiFiEditor => {
                spawn(&self.config.edit_wifi);
            }
            Command::SpawnBluetoothEditor => {
                spawn(&self.config.edit_bluetooth);
            }
            Command::SpawnSystemMonitor => {
                spawn(&self.config.open_system_monitor);
            }
            Command::ChangeWallpaper => {
                spawn(&self.config.change_wallpaper);
            }

            Command::TriggerTray { uuid } => {
                self.tray.trigger(uuid.as_str());
                schedule!(SessionDBus, self.session_dbus, &mut self.ring);
            }
        }

        self.ring.submit_if_dirty();
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
