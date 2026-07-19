use crate::{
    Event,
    command::Command,
    config::Config,
    event_queue::EventQueue,
    liburing::IoUring,
    modules::{Clock, Control, Cpu, KbMod, Memory, NM, Niri, PW, Timer, Tray, Weather},
    sansio::Satisfy,
    user_data::{ModuleId, UserData},
};
use anyhow::Result;
use libc::sockaddr_un;

pub struct IO {
    ring: IoUring,
    events: EventQueue,

    pub(crate) config: Config,

    timer: Option<Timer>,
    timerbuf: [u8; 8],

    pw: Option<PW>,
    pw_addr: sockaddr_un,

    nm: Option<NM>,
    nm_addr: sockaddr_un,

    weather: Option<Weather>,
    weather_addr: sockaddr_un,

    cpu: Option<Cpu>,
    memory: Option<Memory>,

    kb_mod: Option<KbMod>,
    kb_mod_addr: sockaddr_un,

    niri: Option<Niri>,
    niri_addr: sockaddr_un,

    tray: Option<Tray>,
    tray_addr: sockaddr_un,

    control: Control,

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

        let nm = NM::new();
        let nm_addr = NM::address()?;

        let cpu = Cpu::new()?;

        let memory = Memory::new()?;

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

        let control = Control::new()?;

        Ok(Self {
            ring,
            events,

            config,

            timer: Some(timer),
            timerbuf,

            pw_addr,
            pw: Some(pw),

            nm_addr,
            nm: Some(nm),

            weather: Some(weather),
            weather_addr,

            cpu: Some(cpu),
            memory: Some(memory),

            kb_mod: Some(kb_mod),
            kb_mod_addr,

            niri: Some(niri),
            niri_addr,

            tray: Some(tray),
            tray_addr,

            control,

            on_event,
            running: true,
        })
    }

    pub(crate) fn start(&mut self) {
        if let Some(timer) = &mut self.timer {
            schedule_timer(timer, &mut self.ring, &mut self.timerbuf);
        }

        if let Some(nm) = &mut self.nm {
            schedule_nm(nm, &mut self.ring, &self.nm_addr);
        }

        if let Some(pw) = &mut self.pw {
            schedule_pw(pw, &mut self.ring, &self.pw_addr);
        }

        if let Some(cpu) = &mut self.cpu {
            schedule_cpu(cpu, &mut self.ring);
        }

        if let Some(memory) = &mut self.memory {
            schedule_memory(memory, &mut self.ring);
        }

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

        schedule_control(&self.control, &mut self.ring);

        self.ring.submit_if_dirty();
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
                ModuleId::Control => self.satisfy_control(satisfy),
                ModuleId::Cpu => self.satisfy_cpu(satisfy),
                ModuleId::Memory => self.satisfy_memory(satisfy),
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

fn schedule_cpu(cpu: &mut Cpu, ring: &mut IoUring) {
    if let Some(wants) = cpu.wants() {
        log::trace!(target: "Cpu", "{wants:?}");
        core::assert_matches!(cpu.wants(), None);
        ring.schedule(ModuleId::Cpu, wants);
    }
}

fn schedule_memory(memory: &mut Memory, ring: &mut IoUring) {
    if let Some(wants) = memory.wants() {
        log::trace!(target: "Memory", "{wants:?}");
        core::assert_matches!(memory.wants(), None);
        ring.schedule(ModuleId::Memory, wants);
    }
}

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

fn schedule_control(control: &Control, ring: &mut IoUring) {
    let wants = control.wants();
    log::trace!(target: "Control", "{wants:?}");
    ring.schedule(ModuleId::Control, wants);
}

fn schedule_timer(timer: &mut Timer, ring: &mut IoUring, buf: &mut [u8; 8]) {
    if let Some(wants) = timer.wants(buf) {
        log::trace!(target: "Timer", "{wants:?}");
        core::assert_matches!(timer.wants(buf), None);
        ring.schedule(ModuleId::Timer, wants);
    }
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

                if let Some(cpu) = &mut self.cpu {
                    cpu.tick();
                    schedule_cpu(cpu, &mut self.ring);
                }

                if let Some(memory) = &mut self.memory {
                    memory.tick();
                    schedule_memory(memory, &mut self.ring);
                }
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
    fn satisfy_control(&mut self, satisfy: Satisfy) {
        match Control::satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_control(&self.control, &mut self.ring),
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

impl IO {
    fn satisfy_cpu(&mut self, satisfy: Satisfy) {
        let Some(cpu) = &mut self.cpu else {
            return;
        };

        match cpu.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_cpu(cpu, &mut self.ring),
            Err(err) => {
                log::error!(target: "Cpu", "{err:?}");
                self.cpu = None;
            }
        }
    }
}

impl IO {
    fn satisfy_memory(&mut self, satisfy: Satisfy) {
        let Some(memory) = &mut self.memory else {
            return;
        };

        match memory.satisfy(satisfy, &mut self.events) {
            Ok(()) => schedule_memory(memory, &mut self.ring),
            Err(err) => {
                log::error!(target: "Memory", "{err:?}");
                self.memory = None;
            }
        }
    }
}

fn spawn(cmd: &str) {
    if let Err(err) = crate::utils::spawn(cmd) {
        log::error!("{err:?}");
    }
}
