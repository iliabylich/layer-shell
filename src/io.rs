use crate::{
    Event,
    actor::WantsSatisfy,
    command::Command,
    config::Config,
    event_queue::EventQueue,
    external::sockaddr_un,
    liburing::IoUring,
    modules::{
        CPU, Clock, Control, ControlRequest, KbMod, Memory, NM, Niri, PW, SessionDBus, Timer, Tray,
        Weather,
    },
    sansio::Satisfy,
    user_data::{ModuleId, UserData},
    utils::{HeapBlob, dbus::queue::SessionDBusQueue},
};
use anyhow::Result;

pub struct IO {
    ring: IoUring,
    events: EventQueue,

    pub(crate) config: Config,

    timer: Option<Timer>,
    timerbuf: [u8; 8],

    session_dbus: SessionDBus,
    session_dbus_addr: sockaddr_un,
    session_dbus_readbuf: HeapBlob,
    session_dbus_queue: SessionDBusQueue,

    pw: Option<PW>,
    pw_addr: sockaddr_un,

    nm: Option<NM>,
    nm_addr: sockaddr_un,

    weather: Option<Weather>,
    weather_addr: sockaddr_un,

    cpu: CPU,
    memory: Memory,

    kb_mod: Option<KbMod>,
    kb_mod_addr: sockaddr_un,

    niri: Option<Niri>,
    niri_addr: sockaddr_un,

    tray: Option<Tray>,
    tray_addr: sockaddr_un,

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

        let timer = Timer::new()?;
        let timerbuf = [0; 8];

        let session_dbus = SessionDBus::new();
        let session_dbus_addr = SessionDBus::address()?;
        let session_dbus_readbuf = HeapBlob::new(400 * 1_024)?;
        let mut session_dbus_queue = SessionDBusQueue::new()?;
        Control::init(&mut session_dbus_queue)?;

        let nm = NM::new();
        let nm_addr = NM::address()?;

        let cpu = CPU::new();

        let memory = Memory::new();

        let kb_mod = KbMod::new();
        let kb_mod_addr = KbMod::address()?;

        let niri = Niri::new()?;
        let niri_addr = Niri::address()?;

        let pw = PW::new();
        let pw_addr = PW::address()?;

        let weather = Weather::new();
        let weather_addr = Weather::address()?;

        let tray = Tray::new();
        let tray_addr = Tray::address()?;

        Ok(Self {
            ring,
            events,

            config,

            timer: Some(timer),
            timerbuf,

            session_dbus,
            session_dbus_addr,
            session_dbus_readbuf,
            session_dbus_queue,

            pw_addr,
            pw: Some(pw),

            nm_addr,
            nm: Some(nm),

            weather: Some(weather),
            weather_addr,

            cpu,
            memory,

            kb_mod: Some(kb_mod),
            kb_mod_addr,

            niri: Some(niri),
            niri_addr,

            tray: Some(tray),
            tray_addr: tray_addr,

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

        schedule_cpu(&mut self.cpu, &mut self.ring)?;

        schedule_memory(&mut self.memory, &mut self.ring)?;

        if let Some(weather) = &mut self.weather {
            schedule_weather(weather, &mut self.ring, &self.weather_addr);
        }

        if let Some(kb_mod) = &mut self.kb_mod {
            schedule_kb_mod(kb_mod, &mut self.ring, &self.kb_mod_addr);
        }

        if let Some(niri) = &mut self.niri {
            schedule_niri(niri, &mut self.ring, &self.niri_addr);
        }

        if let Some(tray) = &mut self.tray {
            schedule_tray(tray, &mut self.ring, &self.tray_addr);
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
                ModuleId::Weather => self.satisfy_weather(satisfy),
                ModuleId::KbMod => self.satisfy_kb_mod(satisfy),
                ModuleId::NM => self.satisfy_nm(satisfy),
                ModuleId::PW => self.satisfy_pw(satisfy),
                ModuleId::Niri => self.satisfy_niri(satisfy),
                ModuleId::Tray => self.satisfy_tray(satisfy),
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

            Command::TriggerTray { service, id } => {
                if let Some(tray) = &mut self.tray
                    && let Some(wants) = tray.wants_trigger(service, id)
                {
                    log::error!("tray trigger: {service:?} - {id}");
                    self.ring.schedule(ModuleId::Tray, wants);
                }
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

fn schedule_kb_mod(kb_mod: &mut KbMod, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = kb_mod.wants(addr) {
        log::trace!(target: "KbMod", "{wants:?}");
        core::assert_matches!(kb_mod.wants(addr), None);
        ring.schedule(ModuleId::KbMod, wants);
    }
}

fn schedule_weather(weather: &mut Weather, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = weather.wants(addr) {
        log::trace!(target: "Weather", "{wants:?}");
        core::assert_matches!(weather.wants(addr), None);
        ring.schedule(ModuleId::Weather, wants);
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

fn schedule_tray(tray: &mut Tray, ring: &mut IoUring, addr: &sockaddr_un) {
    if let Some(wants) = tray.wants(addr) {
        log::trace!(target: "Tray", "{wants:?}");
        core::assert_matches!(tray.wants(addr), None);
        ring.schedule(ModuleId::Tray, wants);
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
            Ok(Some(_tick)) => {
                schedule_timer(timer, &mut self.ring, &mut self.timerbuf);

                Clock::tick(&mut self.events)?;

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
    fn satisfy_tray(&mut self, satisfy: Satisfy) {
        let Some(tray) = &mut self.tray else {
            return;
        };

        match tray.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_tray(tray, &mut self.ring, &self.tray_addr),
            Err(err) => {
                log::error!(target: "Tray", "{err:?}");
                self.tray = None;
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
    fn satisfy_weather(&mut self, satisfy: Satisfy) {
        let Some(weather) = &mut self.weather else {
            return;
        };

        match weather.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_weather(weather, &mut self.ring, &self.weather_addr),
            Err(err) => {
                log::error!(target: "Weather", "{err:?}");
                self.weather = None;
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
