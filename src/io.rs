use crate::{
    Event,
    actor::WantsSatisfy,
    command::Command,
    config::Config,
    event_queue::EventQueue,
    external::{sockaddr_in, sockaddr_un},
    liburing::IoUring,
    modules::{
        CPU, Clock, Control, ControlRequest, KbMod, Location, Memory, NM, Niri, PW, SessionDBus,
        Timer, Tray, Weather,
    },
    sansio::{DNS, OpenSslContext, Satisfy},
    user_data::{ModuleId, UserData},
    utils::{HeapBlob, dbus::queue::SessionDBusQueue},
};
use anyhow::{Context as _, Result};

pub struct IO {
    ring: IoUring,
    events: EventQueue,
    openssl_ctx: OpenSslContext,
    dns_server_addr: sockaddr_in,

    pub(crate) config: Config,

    timer: Option<Timer>,
    timerbuf: [u8; 8],

    session_dbus: SessionDBus,
    session_dbus_addr: sockaddr_un,
    session_dbus_readbuf: HeapBlob,
    session_dbus_queue: SessionDBusQueue,
    tray: Tray,

    pw: Option<PW>,
    pw_addr: sockaddr_un,

    nm: Option<NM>,
    nm_addr: sockaddr_un,

    location_dns: DNS,
    location_addr: Option<sockaddr_in>,
    location: Option<Location>,
    latlng: Option<(f64, f64)>,

    weather_dns: DNS,
    weather_addr: Option<sockaddr_in>,
    weather: Option<Weather>,

    cpu: CPU,
    memory: Memory,

    kb_mod: Option<KbMod>,
    kb_mod_addr: sockaddr_un,

    niri: Option<Niri>,
    niri_addr: sockaddr_un,

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

        let ring = IoUring::new(10, 0);
        let events = EventQueue::new();
        let openssl_ctx = OpenSslContext::new()?;
        let dns_server_addr = DNS::address();

        let timer = Timer::new()?;
        let timerbuf = [0; 8];

        let session_dbus = SessionDBus::new();
        let session_dbus_addr = SessionDBus::address()?;
        let session_dbus_readbuf = HeapBlob::new(400 * 1_024)?;
        let mut session_dbus_queue = SessionDBusQueue::new()?;
        let tray = Tray::new(&mut session_dbus_queue)?;
        Control::init(&mut session_dbus_queue)?;

        let nm = NM::new();
        let nm_addr = NM::address()?;

        let location_dns = DNS::new(Location::HOST);

        let weather_dns = DNS::new(Weather::HOST);

        let cpu = CPU::new();

        let memory = Memory::new();

        let kb_mod = KbMod::new();
        let kb_mod_addr = KbMod::address()?;

        let niri = Niri::new()?;
        let niri_addr = Niri::address()?;

        let pw = PW::new();
        let pw_addr = PW::address()?;

        Ok(Self {
            ring,
            events,
            openssl_ctx,
            dns_server_addr,

            config,

            timer: Some(timer),
            timerbuf,

            session_dbus,
            session_dbus_addr,
            session_dbus_readbuf,
            session_dbus_queue,
            tray,

            pw_addr,
            pw: Some(pw),

            nm_addr,
            nm: Some(nm),

            location_dns,
            location_addr: None,
            location: None,
            latlng: None,

            weather_dns,
            weather_addr: None,
            weather: None,

            cpu,
            memory,

            kb_mod: Some(kb_mod),
            kb_mod_addr,

            niri: Some(niri),
            niri_addr,

            on_event,
            running: true,
        })
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        if let Some(timer) = &mut self.timer {
            schedule_timer(timer, &mut self.ring, &mut self.timerbuf);
        }

        schedule_session_dbus(
            &mut self.session_dbus,
            self.session_dbus_readbuf.as_slice(),
            &self.session_dbus_addr,
            &self.session_dbus_queue,
            &mut self.ring,
        );

        if let Some(nm) = &mut self.nm {
            schedule_nm(nm, &mut self.ring, &self.nm_addr);
        }

        if let Some(pw) = &mut self.pw {
            schedule_pw(pw, &mut self.ring, &self.pw_addr);
        }

        schedule_location_dns(
            &mut self.location_dns,
            &mut self.ring,
            &self.dns_server_addr,
        )?;

        schedule_cpu(&mut self.cpu, &mut self.ring)?;

        schedule_memory(&mut self.memory, &mut self.ring)?;

        if let Some(kb_mod) = &mut self.kb_mod {
            schedule_kb_mod(kb_mod, &mut self.ring, &self.kb_mod_addr);
        }

        if let Some(niri) = &mut self.niri {
            schedule_niri(niri, &mut self.ring, &self.niri_addr);
        }

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
                ModuleId::LocationDNS => self.satisfy_location_dns(satisfy)?,
                ModuleId::Location => self.satisfy_location(satisfy)?,
                ModuleId::WeatherDNS => self.satisfy_weather_dns(satisfy)?,
                ModuleId::Weather => self.satisfy_weather(satisfy)?,
                ModuleId::KbMod => self.satisfy_kb_mod(satisfy),
                ModuleId::NM => self.satisfy_nm(satisfy),
                ModuleId::PW => self.satisfy_pw(satisfy),
                ModuleId::Niri => self.satisfy_niri(satisfy),
                ModuleId::SessionDBus => self.satisfy_session_dbus(satisfy),
                ModuleId::CPU => self.satisfy_cpu(satisfy)?,
                ModuleId::Memory => self.satisfy_memory(satisfy)?,
                ModuleId::Timer => self.satisfy_timer(satisfy)?,
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty();

        while let Some(event) = self.events.pop_front() {
            log::info!(target: "IO", "{event:?}");
            let (callback, data) = self.on_event;
            (callback)(&event, data);
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
                    self.session_dbus_readbuf.as_slice(),
                    &self.session_dbus_addr,
                    &self.session_dbus_queue,
                    &mut self.ring,
                );
            }
        }

        self.ring.submit_if_dirty();
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.ring.fd()
    }
}

macro_rules! generate_simple_schedule_impl {
    ($fn:ident, $module:ident) => {
        fn $fn(module: &mut $module, ring: &mut IoUring) -> Result<()> {
            if let Some(wants) = module.wants() {
                ring.schedule(ModuleId::$module, wants);
            };
            Ok(())
        }
    };
}

generate_simple_schedule_impl!(schedule_cpu, CPU);
generate_simple_schedule_impl!(schedule_memory, Memory);

fn schedule_location_dns(dns: &mut DNS, ring: &mut IoUring, dns_addr: &sockaddr_in) -> Result<()> {
    if let Some(wants) = dns.try_wants(dns_addr)? {
        log::trace!(target: "LocationDNS", "{wants:?}");
        core::assert_matches!(dns.try_wants(dns_addr), Ok(None));
        ring.schedule(ModuleId::LocationDNS, wants);
    }
    Ok(())
}

fn schedule_location(
    location: &mut Location,
    ring: &mut IoUring,
    remote_server_addr: &sockaddr_in,
) {
    if let Some(wants) = location.wants(remote_server_addr) {
        log::trace!(target: "Location", "{wants:?}");
        core::assert_matches!(location.wants(remote_server_addr), None);
        ring.schedule(ModuleId::Location, wants);
    }
}

fn schedule_weather_dns(dns: &mut DNS, ring: &mut IoUring, dns_addr: &sockaddr_in) -> Result<()> {
    if let Some(wants) = dns.try_wants(dns_addr)? {
        log::trace!(target: "WeatherDNS", "{wants:?}");
        core::assert_matches!(dns.try_wants(dns_addr), Ok(None));
        ring.schedule(ModuleId::WeatherDNS, wants);
    }
    Ok(())
}

fn schedule_weather(weather: &mut Weather, ring: &mut IoUring, remote_server_addr: &sockaddr_in) {
    if let Some(wants) = weather.wants(remote_server_addr) {
        log::trace!(target: "Weather", "{wants:?}");
        core::assert_matches!(weather.wants(remote_server_addr), None);
        ring.schedule(ModuleId::Weather, wants);
    }
}

fn schedule_kb_mod(kb_mod: &mut KbMod, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = kb_mod.wants(addr) {
        log::trace!(target: "KbMod", "{wants:?}");
        core::assert_matches!(kb_mod.wants(addr), None);
        ring.schedule(ModuleId::KbMod, wants);
    }
}

fn schedule_nm(nm: &mut NM, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = nm.wants(addr) {
        log::trace!(target: "NM", "{wants:?}");
        core::assert_matches!(nm.wants(addr), None);
        ring.schedule(ModuleId::NM, wants);
    }
}

fn schedule_pw(pw: &mut PW, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = pw.wants(addr) {
        log::trace!(target: "pw", "{wants:?}");
        core::assert_matches!(pw.wants(addr), None);
        ring.schedule(ModuleId::PW, wants);
    }
}

fn schedule_niri(niri: &mut Niri, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = niri.wants(addr) {
        log::trace!(target: "Niri", "{wants:?}");
        core::assert_matches!(niri.wants(addr), None);
        ring.schedule(ModuleId::Niri, wants);
    }
}

fn schedule_timer(timer: &mut Timer, ring: &mut IoUring, buf: &mut [u8; 8]) {
    if let Some(wants) = timer.wants(buf) {
        log::trace!(target: "Timer", "{wants:?}");
        core::assert_matches!(timer.wants(buf), None);
        ring.schedule(ModuleId::Timer, wants);
    }
}

fn schedule_session_dbus(
    module: &mut SessionDBus,
    readbuf: &mut [u8],
    addr: &sockaddr_un,
    queue: &SessionDBusQueue,
    ring: &mut IoUring,
) {
    let Some(wants) = module.wants(readbuf, queue, addr) else {
        return;
    };
    log::trace!(target: "SessionDBus", "{wants:?}");
    core::assert_matches!(module.wants(readbuf, queue, addr), None);
    ring.schedule(ModuleId::SessionDBus, wants);
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
                schedule_timer(timer, &mut self.ring, &mut self.timerbuf);

                Clock::tick(&mut self.events)?;

                if let Some(weather) = &mut self.weather
                    && let Some((lat, lng)) = self.latlng
                {
                    weather.tick(tick, lat, lng, &self.openssl_ctx)?;
                    schedule_weather(weather, &mut self.ring, &self.dns_server_addr);
                }

                self.cpu.tick();
                schedule_cpu(&mut self.cpu, &mut self.ring)?;

                self.memory.tick();
                schedule_memory(&mut self.memory, &mut self.ring)?;
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
    fn satisfy_niri(&mut self, satisfy: Satisfy) {
        let Some(niri) = &mut self.niri else {
            return;
        };

        match niri.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_niri(niri, &mut self.ring, &self.niri_addr),
            Err(err) => {
                log::error!(target: "Niri", "{err:?}");
                self.niri = None;
            }
        }
    }
}

impl IO {
    fn satisfy_kb_mod(&mut self, satisfy: Satisfy) {
        let Some(kb_mod) = &mut self.kb_mod else {
            return;
        };

        match kb_mod.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_kb_mod(kb_mod, &mut self.ring, &self.kb_mod_addr),
            Err(err) => {
                log::error!(target: "KbMod", "{err:?}");
                self.kb_mod = None;
            }
        }
    }
}

impl IO {
    fn satisfy_nm(&mut self, satisfy: Satisfy) {
        let Some(nm) = &mut self.nm else {
            return;
        };

        match nm.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_nm(nm, &mut self.ring, &self.nm_addr),
            Err(err) => {
                log::error!(target: "NM", "{err:?}");
                self.nm = None;
            }
        }
    }
}

impl IO {
    fn satisfy_pw(&mut self, satisfy: Satisfy) {
        let Some(pw) = &mut self.pw else {
            return;
        };

        match pw.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_pw(pw, &mut self.ring, &self.pw_addr),
            Err(err) => {
                log::error!(target: "PW", "{err:?}");
                self.pw = None;
            }
        }
    }
}

impl IO {
    fn satisfy_location_dns(&mut self, satisfy: Satisfy) -> Result<()> {
        match self.location_dns.try_satisfy(satisfy)? {
            Some(addr) => {
                let location_addr = self.location_addr.insert(addr);
                let location = self.location.insert(Location::new(&self.openssl_ctx)?);
                schedule_location(location, &mut self.ring, location_addr);
            }
            None => {
                schedule_location_dns(
                    &mut self.location_dns,
                    &mut self.ring,
                    &self.dns_server_addr,
                )?;
            }
        }
        Ok(())
    }
    fn satisfy_location(&mut self, satisfy: Satisfy) -> Result<()> {
        let Some(location) = &mut self.location else {
            return Ok(());
        };

        match location.satisfy(satisfy, &mut self.events)? {
            Some((lat, lng)) => {
                self.latlng = Some((lat, lng));
                schedule_weather_dns(&mut self.weather_dns, &mut self.ring, &self.dns_server_addr)?;
            }
            None => {
                schedule_location(
                    location,
                    &mut self.ring,
                    self.location_addr.as_ref().context("no location_addr")?,
                );
            }
        }
        Ok(())
    }
}

impl IO {
    fn satisfy_weather_dns(&mut self, satisfy: Satisfy) -> Result<()> {
        match self.weather_dns.try_satisfy(satisfy)? {
            Some(addr) => {
                let weather_addr = self.weather_addr.insert(addr);
                let (lat, lng) = self.latlng.context("no latlng")?;
                let weather = self
                    .weather
                    .insert(Weather::new(lat, lng, &self.openssl_ctx)?);
                schedule_weather(weather, &mut self.ring, weather_addr);
            }
            None => {
                schedule_weather_dns(&mut self.weather_dns, &mut self.ring, &self.dns_server_addr)?;
            }
        }
        Ok(())
    }
    fn satisfy_weather(&mut self, satisfy: Satisfy) -> Result<()> {
        let Some(weather) = &mut self.weather else {
            return Ok(());
        };

        match weather.satisfy(satisfy, &mut self.events) {
            Ok(()) => {
                schedule_weather(
                    weather,
                    &mut self.ring,
                    self.weather_addr.as_ref().context("no weather_addr")?,
                );
                Ok(())
            }
            Err(err) => {
                log::error!(target: "Weather", "{err:?}");
                self.weather = None;
                Ok(())
            }
        }
    }
}

generate_simple_satisfy_impl!(satisfy_cpu, cpu, schedule_cpu);
generate_simple_satisfy_impl!(satisfy_memory, memory, schedule_memory);

impl IO {
    fn satisfy_session_dbus(&mut self, satisfy: Satisfy) {
        let message = self.session_dbus.satisfy(
            satisfy,
            self.session_dbus_readbuf.as_slice(),
            &mut self.session_dbus_queue,
        );

        if let Some(message) = message {
            self.tray
                .handle(message, &mut self.events, &mut self.session_dbus_queue);

            if let Some(req) = Control::handle(message, &mut self.session_dbus_queue) {
                self.on_control_req(req);
            }
        }

        schedule_session_dbus(
            &mut self.session_dbus,
            self.session_dbus_readbuf.as_slice(),
            &self.session_dbus_addr,
            &self.session_dbus_queue,
            &mut self.ring,
        );
    }
}

fn spawn(cmd: &str) {
    if let Err(err) = crate::utils::spawn(cmd) {
        log::error!("{err:?}");
    }
}
