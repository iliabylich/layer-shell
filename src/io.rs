use crate::{
    Event,
    actor::WantsSatisfy,
    command::Command,
    config::Config,
    event_queue::EventQueue,
    liburing::IoUring,
    modules::{
        CPU, Clock, Control, ControlRequest, KbMod, Location, Memory, Network, Niri, SessionDBus,
        Sound, SystemDBus, Timer, Tray, Weather,
    },
    sansio::{OpenSslContext, Satisfy},
    user_data::{ModuleId, UserData},
    utils::dbus::queue::{SessionDBusQueue, SystemDBusQueue},
};
use alloc::{vec, vec::Vec};
use anyhow::Result;
use rustix::net::SocketAddrUnix;

pub struct IO {
    ring: IoUring,
    events: EventQueue,
    openssl_ctx: OpenSslContext,

    pub(crate) config: Config,

    timer: Option<Timer>,
    timerbuf: [u8; 8],

    session_dbus: SessionDBus,
    session_dbus_addr: SocketAddrUnix,
    session_dbus_readbuf: Vec<u8>,
    session_dbus_queue: SessionDBusQueue,
    sound: Sound,
    tray: Tray,

    system_dbus: SystemDBus,
    system_dbus_addr: SocketAddrUnix,
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
        extern "C" fn(event: &Event, *mut core::ffi::c_void),
        *mut core::ffi::c_void,
    ),
    running: bool,
}

impl IO {
    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.ring.deinit();
    }

    pub(crate) fn new(
        on_event: (
            extern "C" fn(event: &Event, *mut core::ffi::c_void),
            *mut core::ffi::c_void,
        ),
    ) -> Result<Self> {
        let config = Config::read()?;

        let ring = IoUring::new(10, 0)?;
        let events = EventQueue::new();
        let openssl_ctx = OpenSslContext::new()?;

        let timer = Timer::new()?;
        let timerbuf = [0; 8];

        let session_dbus = SessionDBus::new();
        let session_dbus_addr =
            SocketAddrUnix::new(SessionDBus::address()?).map_err(|errno| anyhow::anyhow!(errno))?;
        let session_dbus_readbuf = vec![0; 400 * 1_024];
        let mut session_dbus_queue = SessionDBusQueue::new()?;
        let sound = Sound::new(&mut session_dbus_queue);
        let tray = Tray::new(&mut session_dbus_queue)?;
        Control::init(&mut session_dbus_queue)?;

        let system_dbus = SystemDBus::new();
        let system_dbus_addr =
            SocketAddrUnix::new(SystemDBus::address()?).map_err(|errno| anyhow::anyhow!(errno))?;
        let system_dbus_readbuf = vec![0; 400 * 1_024];
        let mut system_dbus_queue = SystemDBusQueue::new()?;
        let network = Network::new(&mut system_dbus_queue);

        let location = Location::new(&openssl_ctx)?;

        let weather = Weather::new();

        let cpu = CPU::new();

        let memory = Memory::new();

        let kb_mod = KbMod::new();

        let niri = Niri::new();

        Ok(Self {
            ring,
            events,
            openssl_ctx,

            config,

            timer: Some(timer),
            timerbuf,

            session_dbus,
            session_dbus_addr,
            session_dbus_readbuf,
            session_dbus_queue,
            sound,
            tray,

            system_dbus,
            system_dbus_addr,
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

    pub(crate) fn start(&mut self) -> Result<()> {
        if let Some(timer) = &mut self.timer {
            schedule_timer(timer, &mut self.ring, &mut self.timerbuf)?;
        }

        schedule_session_dbus(
            &mut self.session_dbus,
            &mut self.session_dbus_readbuf,
            &self.session_dbus_addr,
            &self.session_dbus_queue,
            &mut self.ring,
        )?;

        schedule_system_dbus(
            &mut self.system_dbus,
            &mut self.system_dbus_readbuf,
            &self.system_dbus_addr,
            &self.system_dbus_queue,
            &mut self.ring,
        )?;

        schedule_location(&mut self.location, &mut self.ring)?;

        schedule_cpu(&mut self.cpu, &mut self.ring)?;

        schedule_memory(&mut self.memory, &mut self.ring)?;

        schedule_kb_mod(&mut self.kb_mod, &mut self.ring)?;

        schedule_niri(&mut self.niri, &mut self.ring)?;

        self.ring.submit_if_dirty()?;

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

        while let Some(cqe) = self.ring.try_get_cqe()? {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::try_from(user_data)?;
            let satisfy = Satisfy::new(op, res);
            log::trace!(target: module_id.as_str(), "Satisfy {satisfy:?}");

            match module_id {
                ModuleId::Location => self.satisfy_location(satisfy)?,
                ModuleId::Weather => self.satisfy_weather(satisfy)?,
                ModuleId::KbMod => self.satisfy_kb_mod(satisfy)?,
                ModuleId::Niri => self.satisfy_niri(satisfy)?,
                ModuleId::SessionDBus => self.satisfy_session_dbus(satisfy)?,
                ModuleId::SystemDBus => self.satisfy_system_dbus(satisfy)?,
                ModuleId::CPU => self.satisfy_cpu(satisfy)?,
                ModuleId::Memory => self.satisfy_memory(satisfy)?,
                ModuleId::Timer => self.satisfy_timer(satisfy)?,
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty()?;

        while let Some(event) = self.events.pop_front() {
            log::info!(target: "IO", "{event:?}");
            let (callback, data) = self.on_event;
            (callback)(&event, data);
        }

        Ok(())
    }

    pub(crate) fn wait_readable(&mut self) -> Result<()> {
        self.ring.submit_and_wait(1)?;
        Ok(())
    }

    pub(crate) fn process_command(&mut self, cmd: Command) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        match cmd {
            Command::Lock => spawn(self.config.lock.as_str()),
            Command::Reboot => spawn(self.config.reboot.as_str()),
            Command::Shutdown => spawn(self.config.shutdown.as_str()),
            Command::Logout => spawn(self.config.logout.as_str()),
            Command::SpawnWiFiEditor => spawn(self.config.edit_wifi.as_str()),
            Command::SpawnBluetoothEditor => spawn(self.config.edit_bluetooth.as_str()),
            Command::SpawnSystemMonitor => spawn(self.config.open_system_monitor.as_str()),
            Command::ChangeWallpaper => spawn(self.config.change_wallpaper.as_str()),

            Command::TriggerTray { uuid } => {
                self.tray
                    .trigger(uuid.as_str(), &mut self.session_dbus_queue);
                schedule_session_dbus(
                    &mut self.session_dbus,
                    &mut self.session_dbus_readbuf,
                    &self.session_dbus_addr,
                    &self.session_dbus_queue,
                    &mut self.ring,
                )?;
            }
        }

        self.ring.submit_if_dirty()?;
        Ok(())
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.ring.fd()
    }
}

macro_rules! generate_simple_schedule_impl {
    ($fn:ident, $module:ident) => {
        fn $fn(module: &mut $module, ring: &mut IoUring) -> Result<()> {
            if let Some(wants) = module.wants() {
                ring.schedule(ModuleId::$module, wants)?;
            };
            Ok(())
        }
    };
}

fn schedule_timer(timer: &mut Timer, ring: &mut IoUring, buf: &mut [u8; 8]) -> Result<()> {
    if let Some(wants) = timer.wants(buf) {
        log::trace!(target: "Timer", "{wants:?}");
        core::assert_matches!(timer.wants(buf), None);
        ring.schedule(ModuleId::Timer, wants)?;
    }
    Ok(())
}

generate_simple_schedule_impl!(schedule_location, Location);
generate_simple_schedule_impl!(schedule_weather, Weather);
generate_simple_schedule_impl!(schedule_cpu, CPU);
generate_simple_schedule_impl!(schedule_memory, Memory);
generate_simple_schedule_impl!(schedule_kb_mod, KbMod);
generate_simple_schedule_impl!(schedule_niri, Niri);

fn schedule_session_dbus(
    module: &mut SessionDBus,
    readbuf: &mut [u8],
    addr: &SocketAddrUnix,
    queue: &SessionDBusQueue,
    ring: &mut IoUring,
) -> Result<()> {
    let Some(wants) = module.wants(readbuf, queue, addr) else {
        return Ok(());
    };
    log::trace!(target: "SessionDBus", "{wants:?}");
    core::assert_matches!(module.wants(readbuf, queue, addr), None);
    ring.schedule(ModuleId::SessionDBus, wants)?;
    Ok(())
}
fn schedule_system_dbus(
    module: &mut SystemDBus,
    readbuf: &mut [u8],
    addr: &SocketAddrUnix,
    queue: &SystemDBusQueue,
    ring: &mut IoUring,
) -> Result<()> {
    let Some(wants) = module.wants(readbuf, queue, addr) else {
        return Ok(());
    };
    log::trace!(target: "SystemDBus", "{wants:?}");
    core::assert_matches!(module.wants(readbuf, queue, addr), None);
    ring.schedule(ModuleId::SystemDBus, wants)?;
    Ok(())
}

macro_rules! generate_simple_satisfy_impl {
    ($fn:ident, $module:ident, $schedule:ident) => {
        impl IO {
            fn $fn(&mut self, satisfy: Satisfy) -> Result<()> {
                self.$module.satisfy(satisfy, &mut self.events);
                $schedule(&mut self.$module, &mut self.ring)?;
                Ok(())
            }
        }
    };
}

impl IO {
    fn satisfy_timer(&mut self, satisfy: Satisfy) -> Result<()> {
        let Some(timer) = &mut self.timer else {
            return Ok(());
        };

        match timer.satisfy(satisfy, self.timerbuf) {
            Ok(Some(tick)) => {
                schedule_timer(timer, &mut self.ring, &mut self.timerbuf)?;

                Clock::tick(&mut self.events)?;

                self.weather.tick(tick, &self.openssl_ctx)?;
                schedule_weather(&mut self.weather, &mut self.ring)?;

                self.cpu.tick();
                schedule_cpu(&mut self.cpu, &mut self.ring)?;

                self.memory.tick();
                schedule_memory(&mut self.memory, &mut self.ring)?;

                self.sound.tick(tick, &mut self.session_dbus_queue);
                schedule_session_dbus(
                    &mut self.session_dbus,
                    &mut self.session_dbus_readbuf,
                    &self.session_dbus_addr,
                    &self.session_dbus_queue,
                    &mut self.ring,
                )?;
            }
            Ok(None) => {}
            Err(err) => {
                log::error!(target: "Timer", "{err:?}");
                self.timer = None;
            }
        }

        Ok(())
    }
}

impl IO {
    fn satisfy_location(&mut self, satisfy: Satisfy) -> Result<()> {
        if let Some((lat, lng)) = self.location.satisfy(satisfy, &mut self.events) {
            self.weather.start(lat, lng, &self.openssl_ctx)?;
            schedule_weather(&mut self.weather, &mut self.ring)?;
        } else {
            schedule_location(&mut self.location, &mut self.ring)?;
        }
        Ok(())
    }
}

generate_simple_satisfy_impl!(satisfy_weather, weather, schedule_weather);
generate_simple_satisfy_impl!(satisfy_cpu, cpu, schedule_cpu);
generate_simple_satisfy_impl!(satisfy_memory, memory, schedule_memory);
generate_simple_satisfy_impl!(satisfy_kb_mod, kb_mod, schedule_kb_mod);
generate_simple_satisfy_impl!(satisfy_niri, niri, schedule_niri);

impl IO {
    fn satisfy_session_dbus(&mut self, satisfy: Satisfy) -> Result<()> {
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
            &self.session_dbus_addr,
            &self.session_dbus_queue,
            &mut self.ring,
        )?;
        Ok(())
    }
}

impl IO {
    fn satisfy_system_dbus(&mut self, satisfy: Satisfy) -> Result<()> {
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
            &self.system_dbus_addr,
            &self.system_dbus_queue,
            &mut self.ring,
        )?;
        Ok(())
    }
}

fn spawn(cmd: &str) {
    if let Err(err) = crate::utils::spawn(cmd) {
        log::error!("{err:?}");
    }
}
