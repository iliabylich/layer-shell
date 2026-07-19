use crate::{
    Event,
    command::Command,
    config::Config,
    emitter::Emitter,
    liburing::IoUring,
    modules::{Clock, Control, Cpu, KbMod, Memory, NM, Niri, PW, Timer, Tray, Weather},
    sansio::Satisfy,
    user_data::{ModuleId, UserData},
    utils::spawn,
};
use anyhow::Result;
use libc::sockaddr_un;

pub struct IO {
    ring: IoUring,

    pub(crate) config: Config,

    timer: Option<Timer>,
    timerbuf: [u8; 8],

    clock: Option<Clock>,

    pw: Option<PW>,
    pw_addr: sockaddr_un,

    nm: Option<NM>,
    nm_addr: sockaddr_un,

    weather: Option<Weather>,
    weather_addr: sockaddr_un,

    cpu: Option<Cpu>,
    cpubuf: [u8; 1_024],

    memory: Option<Memory>,
    memorybuf: [u8; 1_024],

    kb_mod: Option<KbMod>,
    kb_mod_addr: sockaddr_un,

    niri: Option<Niri>,
    niri_addr: sockaddr_un,

    tray: Option<Tray>,
    tray_addr: sockaddr_un,

    control: Control,

    running: bool,
}

impl IO {
    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.ring.deinit();
    }

    pub(crate) fn new(
        callback: extern "C" fn(event: &Event, *mut core::ffi::c_void),
        data: *mut core::ffi::c_void,
    ) -> Result<Self> {
        let config = Config::read()?;
        let emitter = Emitter::new(callback, data);

        let ring = IoUring::new(10, 0);

        let timer = Timer::new()?;
        let timerbuf = [0; 8];

        let clock = Clock::new(emitter);

        let nm = NM::new(emitter);
        let nm_addr = NM::address()?;

        let cpu = Cpu::new(emitter)?;

        let memory = Memory::new(emitter)?;

        let kb_mod = KbMod::new(emitter);
        let kb_mod_addr = KbMod::address()?;

        let niri = Niri::new(emitter)?;
        let niri_addr = Niri::address()?;

        let pw = PW::new(emitter);
        let pw_addr = PW::address()?;

        let weather = Weather::new(emitter);
        let weather_addr = Weather::address()?;

        let tray = Tray::new(emitter);
        let tray_addr = Tray::address()?;

        let control = Control::new(emitter)?;

        Ok(Self {
            ring,

            config,

            timer: Some(timer),
            timerbuf,

            clock: Some(clock),

            pw_addr,
            pw: Some(pw),

            nm_addr,
            nm: Some(nm),

            weather: Some(weather),
            weather_addr,

            cpu: Some(cpu),
            cpubuf: [0; _],

            memory: Some(memory),
            memorybuf: [0; _],

            kb_mod: Some(kb_mod),
            kb_mod_addr,

            niri: Some(niri),
            niri_addr,

            tray: Some(tray),
            tray_addr,

            control,

            running: true,
        })
    }

    pub(crate) fn start(&mut self) {
        IoSlice::<Timer>::schedule(self);
        IoSlice::<NM>::schedule(self);
        IoSlice::<PW>::schedule(self);
        IoSlice::<Cpu>::schedule(self);
        IoSlice::<Memory>::schedule(self);
        IoSlice::<Weather>::schedule(self);
        IoSlice::<KbMod>::schedule(self);
        IoSlice::<Niri>::schedule(self);
        IoSlice::<Tray>::schedule(self);
        IoSlice::<Control>::schedule(self);

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
                ModuleId::Weather => IoSlice::<Weather>::satisfy(self, satisfy),
                ModuleId::KbMod => IoSlice::<KbMod>::satisfy(self, satisfy),
                ModuleId::NM => IoSlice::<NM>::satisfy(self, satisfy),
                ModuleId::PW => IoSlice::<PW>::satisfy(self, satisfy),
                ModuleId::Niri => IoSlice::<Niri>::satisfy(self, satisfy),
                ModuleId::Tray => IoSlice::<Tray>::satisfy(self, satisfy),
                ModuleId::Control => IoSlice::<Control>::satisfy(self, satisfy),
                ModuleId::Cpu => IoSlice::<Cpu>::satisfy(self, satisfy),
                ModuleId::Memory => IoSlice::<Memory>::satisfy(self, satisfy),
                ModuleId::Timer => IoSlice::<Timer>::satisfy(self, satisfy),
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty();

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

trait IoSlice<T> {
    fn schedule(&mut self);
    fn satisfy(&mut self, satisfy: Satisfy);
}

impl IoSlice<Cpu> for IO {
    fn schedule(&mut self) {
        if let Some(cpu) = &mut self.cpu
            && let Some(wants) = cpu.wants(&mut self.cpubuf)
        {
            log::trace!(target: "Cpu", "{wants:?}");
            core::assert_matches!(cpu.wants(&mut self.cpubuf), None);
            self.ring.schedule(ModuleId::Cpu, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(cpu) = &mut self.cpu {
            match cpu.satisfy(satisfy, &self.cpubuf) {
                Ok(()) => IoSlice::<Cpu>::schedule(self),
                Err(err) => {
                    log::error!(target: "Cpu", "{err:?}");
                    self.cpu = None;
                }
            }
        }
    }
}

impl IoSlice<Memory> for IO {
    fn schedule(&mut self) {
        if let Some(memory) = &mut self.memory
            && let Some(wants) = memory.wants(&mut self.memorybuf)
        {
            log::trace!(target: "Memory", "{wants:?}");
            core::assert_matches!(memory.wants(&mut self.memorybuf), None);
            self.ring.schedule(ModuleId::Memory, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(memory) = &mut self.memory {
            match memory.satisfy(satisfy, &self.memorybuf) {
                Ok(()) => IoSlice::<Memory>::schedule(self),
                Err(err) => {
                    log::error!(target: "Memory", "{err:?}");
                    self.memory = None;
                }
            }
        }
    }
}

impl IoSlice<KbMod> for IO {
    fn schedule(&mut self) {
        if let Some(kb_mod) = &mut self.kb_mod
            && let Some(wants) = kb_mod.wants(&self.kb_mod_addr)
        {
            log::trace!(target: "KbMod", "{wants:?}");
            core::assert_matches!(kb_mod.wants(&self.kb_mod_addr), None);
            self.ring.schedule(ModuleId::KbMod, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(kb_mod) = &mut self.kb_mod {
            match kb_mod.satisfy(satisfy) {
                Ok(()) => IoSlice::<KbMod>::schedule(self),
                Err(err) => {
                    log::error!(target: "KbMod", "{err:?}");
                    self.kb_mod = None;
                }
            }
        }
    }
}

impl IoSlice<Weather> for IO {
    fn schedule(&mut self) {
        if let Some(weather) = &mut self.weather
            && let Some(wants) = weather.wants(&self.weather_addr)
        {
            log::trace!(target: "Weather", "{wants:?}");
            core::assert_matches!(weather.wants(&self.weather_addr), None);
            self.ring.schedule(ModuleId::Weather, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(weather) = &mut self.weather {
            match weather.satisfy(satisfy) {
                Ok(()) => IoSlice::<Weather>::schedule(self),
                Err(err) => {
                    log::error!(target: "Weather", "{err:?}");
                    self.weather = None;
                }
            }
        }
    }
}

impl IoSlice<NM> for IO {
    fn schedule(&mut self) {
        if let Some(nm) = &mut self.nm
            && let Some(wants) = nm.wants(&self.nm_addr)
        {
            log::trace!(target: "NM", "{wants:?}");
            core::assert_matches!(nm.wants(&self.nm_addr), None);
            self.ring.schedule(ModuleId::NM, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(nm) = &mut self.nm {
            match nm.satisfy(satisfy) {
                Ok(()) => IoSlice::<NM>::schedule(self),
                Err(err) => {
                    log::error!(target: "NM", "{err:?}");
                    self.nm = None;
                }
            }
        }
    }
}

impl IoSlice<PW> for IO {
    fn schedule(&mut self) {
        if let Some(pw) = &mut self.pw
            && let Some(wants) = pw.wants(&self.pw_addr)
        {
            log::trace!(target: "PW", "{wants:?}");
            core::assert_matches!(pw.wants(&self.pw_addr), None);
            self.ring.schedule(ModuleId::PW, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(pw) = &mut self.pw {
            match pw.satisfy(satisfy) {
                Ok(()) => IoSlice::<PW>::schedule(self),
                Err(err) => {
                    log::error!(target: "PW", "{err:?}");
                    self.pw = None;
                }
            }
        }
    }
}

impl IoSlice<Niri> for IO {
    fn schedule(&mut self) {
        if let Some(niri) = &mut self.niri
            && let Some(wants) = niri.wants(&self.niri_addr)
        {
            log::trace!(target: "Niri", "{wants:?}");
            core::assert_matches!(niri.wants(&self.niri_addr), None);
            self.ring.schedule(ModuleId::Niri, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(niri) = &mut self.niri {
            match niri.satisfy(satisfy) {
                Ok(()) => IoSlice::<Niri>::schedule(self),
                Err(err) => {
                    log::error!(target: "Niri", "{err:?}");
                    self.niri = None;
                }
            }
        }
    }
}

impl IoSlice<Tray> for IO {
    fn schedule(&mut self) {
        if let Some(tray) = &mut self.tray
            && let Some(wants) = tray.wants(&self.tray_addr)
        {
            log::trace!(target: "Tray", "{wants:?}");
            core::assert_matches!(tray.wants(&self.tray_addr), None);
            self.ring.schedule(ModuleId::Tray, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(tray) = &mut self.tray {
            match tray.satisfy(satisfy) {
                Ok(()) => IoSlice::<Tray>::schedule(self),
                Err(err) => {
                    log::error!(target: "Tray", "{err:?}");
                    self.tray = None;
                }
            }
        }
    }
}

impl IoSlice<Control> for IO {
    fn schedule(&mut self) {
        let wants = self.control.wants();
        log::trace!(target: "Control", "{wants:?}");
        self.ring.schedule(ModuleId::Control, wants);
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        match self.control.satisfy(satisfy) {
            Ok(()) => IoSlice::<Control>::schedule(self),
            Err(err) => {
                log::error!(target: "Tray", "{err:?}");
                self.tray = None;
            }
        }
    }
}

impl IoSlice<Timer> for IO {
    fn schedule(&mut self) {
        if let Some(timer) = &mut self.timer
            && let Some(wants) = timer.wants(&mut self.timerbuf)
        {
            log::trace!(target: "Timer", "{wants:?}");
            core::assert_matches!(timer.wants(&mut self.timerbuf), None);
            self.ring.schedule(ModuleId::Timer, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        let Some(timer) = &mut self.timer else {
            return;
        };

        match timer.satisfy(satisfy, self.timerbuf) {
            Ok(Some(_tick)) => {
                IoSlice::<Timer>::schedule(self);

                if let Some(clock) = &self.clock
                    && let Err(err) = clock.tick()
                {
                    log::error!(target: "Clock" , "{err:?}");
                    self.clock = None;
                }

                if let Some(cpu) = &mut self.cpu {
                    cpu.tick();
                    IoSlice::<Cpu>::schedule(self);
                }

                if let Some(memory) = &mut self.memory {
                    memory.tick();
                    IoSlice::<Memory>::schedule(self);
                }
            }
            Ok(None) => {}
            Err(err) => {
                log::error!(target: "Timer", "{err:?}");
                self.timer = None;
            }
        }
    }
}
