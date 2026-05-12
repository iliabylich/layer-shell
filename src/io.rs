use std::os::fd::AsRawFd;

use crate::{
    Event,
    command::Command,
    config::{Config, IOConfig},
    event_queue::EventQueue,
    liburing::IoUring,
    modules::{
        CPU, CapsLock, Clock, Control, ControlRequest, Location, Memory, Network, Niri,
        SessionDBus, Sound, SystemDBus, Timer, Tray, Weather,
    },
    sansio::{Https, Satisfy},
    user_data::{ModuleId, UserData},
    utils::{DedupModule, InfallibleModule},
};
use anyhow::{Context, Result};

pub(crate) struct IO {
    io_uring: IoUring,

    config: Config,
    pub(crate) io_config: *const IOConfig,

    timer: InfallibleModule<DedupModule<Timer>>,

    session_dbus: InfallibleModule<DedupModule<SessionDBus>>,
    sound: Sound,
    tray: Tray,
    system_dbus: InfallibleModule<DedupModule<SystemDBus>>,
    network: Network,

    location: InfallibleModule<DedupModule<Location>>,
    coordinates: Option<(f64, f64)>,
    weather: InfallibleModule<DedupModule<Weather>>,

    cpu: InfallibleModule<DedupModule<CPU>>,
    memory: InfallibleModule<DedupModule<Memory>>,

    caps_lock: InfallibleModule<DedupModule<CapsLock>>,
    niri: InfallibleModule<DedupModule<Niri>>,

    on_event: extern "C" fn(event: *const Event),
    running: bool,
}

macro_rules! schedule {
    ($module:expr, $io_uring:expr) => {{
        let module_id = $module.module_id();

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
        SessionDBus::init();
        SystemDBus::init();
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.io_uring.deinit();
    }

    pub(crate) fn new(on_event: extern "C" fn(event: *const Event)) -> Result<Self> {
        let config = Config::read()?;
        let io_config = IOConfig::new(&config);

        let mut this = Self {
            io_uring: IoUring::new(10, 0),

            config,
            io_config,

            timer: InfallibleModule::new(DedupModule::new(Timer::new())),

            session_dbus: InfallibleModule::new(DedupModule::new(SessionDBus::new())),
            sound: Sound::new(),
            tray: Tray::new(),
            system_dbus: InfallibleModule::new(DedupModule::new(SystemDBus::new())),
            network: Network::new(),

            location: InfallibleModule::new(DedupModule::new(Location::new())),
            coordinates: None,
            weather: InfallibleModule::new(DedupModule::new(Weather::new())),

            cpu: InfallibleModule::new(DedupModule::new(CPU::new())),
            memory: InfallibleModule::new(DedupModule::new(Memory::new())),

            caps_lock: InfallibleModule::new(DedupModule::new(CapsLock::new())),
            niri: InfallibleModule::new(DedupModule::new(Niri::new())),

            on_event,
            running: true,
        };

        this.start();

        Ok(this)
    }

    fn start(&mut self) {
        schedule!(self.timer, &mut self.io_uring);

        schedule!(self.location, &mut self.io_uring);
        schedule!(self.cpu, &mut self.io_uring);
        schedule!(self.memory, &mut self.io_uring);
        schedule!(self.caps_lock, &mut self.io_uring);
        schedule!(self.niri, &mut self.io_uring);

        self.sound.init();
        Control::init();
        self.tray.init();
        schedule!(self.session_dbus, &mut self.io_uring);

        self.network.init();
        schedule!(self.system_dbus, &mut self.io_uring);

        self.io_uring.submit_if_dirty();
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

        while let Some(cqe) = self.io_uring.try_get_cqe() {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::try_from(user_data)?;
            let satisfy = Satisfy::try_from(op)?;
            log::trace!(target: module_id.as_str(), "Satisfy {satisfy:?} {res}");

            macro_rules! satisfy {
                ($module:expr) => {
                    $module.satisfy(satisfy, res)
                };
            }

            match module_id {
                ModuleId::GeoLocation => {
                    if let Some((lat, lng)) = satisfy!(self.location) {
                        self.coordinates = Some((lat, lng));
                        if let Some(weather) = self.weather.as_mut() {
                            weather.setup(lat, lng);
                        }
                        schedule!(self.weather, &mut self.io_uring);
                    } else {
                        schedule!(self.location, &mut self.io_uring);
                    }
                }

                ModuleId::Weather => {
                    satisfy!(self.weather);
                    schedule!(self.weather, &mut self.io_uring);
                }

                ModuleId::CapsLock => {
                    satisfy!(self.caps_lock);
                    schedule!(self.caps_lock, &mut self.io_uring);
                }

                ModuleId::Niri => {
                    satisfy!(self.niri);
                    schedule!(self.niri, &mut self.io_uring);
                }

                ModuleId::SessionDBus => {
                    let message = satisfy!(self.session_dbus);

                    if let Some(message) = message {
                        self.sound.on_message(message);
                        self.tray.on_message(message);

                        if let Some(req) = Control::on_message(message) {
                            Self::on_control_req(req);
                        }
                    }

                    schedule!(self.session_dbus, &mut self.io_uring);
                }

                ModuleId::SystemDBus => {
                    let message = satisfy!(self.system_dbus);

                    if let Some(message) = message {
                        self.network.on_message(message);
                    }

                    schedule!(self.system_dbus, &mut self.io_uring);
                }

                ModuleId::CPU => {
                    satisfy!(self.cpu);
                    schedule!(self.cpu, &mut self.io_uring);
                }
                ModuleId::Memory => {
                    satisfy!(self.memory);
                    schedule!(self.memory, &mut self.io_uring);
                }
                ModuleId::Timer => {
                    if let Some(tick) = satisfy!(self.timer) {
                        schedule!(self.timer, &mut self.io_uring);

                        Clock::tick();

                        self.weather.tick(tick);
                        schedule!(self.weather, &mut self.io_uring);

                        self.cpu.tick(tick);
                        schedule!(self.cpu, &mut self.io_uring);

                        self.memory.tick(tick);
                        schedule!(self.memory, &mut self.io_uring);

                        self.sound.tick(tick);
                        schedule!(self.session_dbus, &mut self.io_uring);
                    }
                }
            }

            self.io_uring.cqe_seen(cqe);
        }

        self.io_uring.submit_if_dirty();

        while let Some(event) = EventQueue::pop_front() {
            log::info!(target: "IO", "{event:?}");
            (self.on_event)(&raw const event);
        }

        Ok(())
    }

    pub(crate) fn wait_readable(&mut self) {
        self.io_uring.submit_and_wait(1);
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
                schedule!(self.session_dbus, &mut self.io_uring);
            }
        }

        self.io_uring.submit_if_dirty();
    }
}

impl AsRawFd for IO {
    fn as_raw_fd(&self) -> i32 {
        self.io_uring.as_raw_fd()
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
