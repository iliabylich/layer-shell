use crate::{
    IoEvent,
    command::Command,
    config::Config,
    emitter::Emitter,
    liburing::IoUring,
    modules::{Clock, Control, Cpu, KbMod, Memory, NM, Niri, PW, Tray, Weather},
    sansio::{Satisfy, TimerFd},
    user_data::{ModuleId, UserData},
    utils::{EnvHelper, FixedSizeBuffer, NlSeparatedBuffer, SpawnHelper},
};
use libc::sockaddr_un;

pub struct IO {
    ring: IoUring,

    pub(crate) config: Config,

    timer: Option<TimerFd>,
    timerbuf: [u8; 8],

    clock: Clock,

    pw: Option<PW>,
    pw_buf: FixedSizeBuffer<{ PW::BUFFER_SIZE }>,
    pw_addr: sockaddr_un,

    nm: Option<NM>,
    nm_buf: FixedSizeBuffer<{ NM::BUFFER_SIZE }>,
    nm_addr: sockaddr_un,

    weather: Option<Weather>,
    weather_buf: FixedSizeBuffer<{ Weather::BUFFER_SIZE }>,
    weather_addr: sockaddr_un,

    cpu: Option<Cpu>,
    cpu_buf: [u8; 1_024],

    memory: Option<Memory>,
    memory_buf: [u8; 1_024],

    kb_mod: Option<KbMod>,
    kb_mod_buf: FixedSizeBuffer<{ KbMod::BUFFER_SIZE }>,
    kb_mod_addr: sockaddr_un,

    niri: Option<Niri>,
    niri_buf: NlSeparatedBuffer,
    niri_addr: Option<sockaddr_un>,

    tray: Option<Tray>,
    tray_buf: FixedSizeBuffer<{ Tray::BUFFER_SIZE }>,
    tray_addr: sockaddr_un,

    control: Option<Control>,
    running: bool,

    home: &'static str,
    #[expect(dead_code)]
    xdg_runtime_dir: &'static str,
    #[expect(dead_code)]
    xdg_config_dir: Option<&'static str>,
}

impl IO {
    pub(crate) fn stop(&mut self) {
        self.running = false;
        self.ring.deinit();
    }

    pub(crate) fn new(
        callback: extern "C" fn(event: &IoEvent, *mut core::ffi::c_void),
        data: *mut core::ffi::c_void,
    ) -> Self {
        log::trace!("Creating IO");
        let home = EnvHelper::home();
        let xdg_runtime_dir = EnvHelper::xdg_runtime_dir();
        let xdg_config_dir = EnvHelper::xdg_config_dir();

        let config = Config::read(xdg_config_dir, home);
        let emitter = Emitter::new(callback, data);

        let ring = IoUring::new(10, 0);

        let timer = TimerFd::new();
        let timerbuf = [0; 8];

        let clock = Clock::new(emitter);

        let nm = NM::new(emitter);
        let nm_addr = NM::ADDRESS;

        let cpu = Cpu::new(emitter);

        let memory = Memory::new(emitter);

        let kb_mod = KbMod::new(emitter);
        let kb_mod_addr = KbMod::ADDRESS;

        let niri = Niri::new(emitter);
        let niri_addr = Niri::address();

        let pw = PW::new(emitter);
        let pw_addr = PW::address(xdg_runtime_dir);

        let weather = Weather::new(emitter);
        let weather_addr = Weather::address(xdg_runtime_dir);

        let tray = Tray::new(emitter);
        let tray_addr = Tray::address(xdg_runtime_dir);

        let control = Control::new(xdg_runtime_dir, emitter);

        Self {
            ring,

            config,

            timer,
            timerbuf,

            clock,

            pw: Some(pw),
            pw_buf: FixedSizeBuffer::new(),
            pw_addr,

            nm: Some(nm),
            nm_buf: FixedSizeBuffer::new(),
            nm_addr,

            weather: Some(weather),
            weather_buf: FixedSizeBuffer::new(),
            weather_addr,

            cpu,
            cpu_buf: [0; _],

            memory,
            memory_buf: [0; _],

            kb_mod: Some(kb_mod),
            kb_mod_buf: FixedSizeBuffer::new(),
            kb_mod_addr,

            niri: Some(niri),
            niri_buf: NlSeparatedBuffer::new(),
            niri_addr,

            tray: Some(tray),
            tray_buf: FixedSizeBuffer::new(),
            tray_addr,

            control,
            running: true,

            home,
            xdg_runtime_dir,
            xdg_config_dir,
        }
    }

    pub(crate) fn start(&mut self) {
        IoSlice::<TimerFd>::schedule(self);
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

    pub(crate) fn handle_readable(&mut self) {
        if !self.running {
            return;
        }

        while let Some(cqe) = self.ring.try_get_cqe() {
            let res = cqe.res();
            let user_data = cqe.user_data();

            let UserData { module_id, op, .. } = UserData::decode(user_data);
            let satisfy = Satisfy::new(op, res, module_id);
            log::trace!("Satisfy {satisfy:?}");

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
                ModuleId::Timer => IoSlice::<TimerFd>::satisfy(self, satisfy),
            }

            self.ring.cqe_seen(cqe);
        }

        self.ring.submit_if_dirty();
    }

    pub(crate) fn wait_readable(&mut self) {
        self.ring.submit_and_wait(1);
    }

    pub(crate) fn process_command(&mut self, cmd: Command) {
        if !self.running {
            return;
        }

        match cmd {
            Command::Lock => self.spawn(self.config.lock.as_str()),
            Command::Reboot => self.spawn(self.config.reboot.as_str()),
            Command::Shutdown => self.spawn(self.config.shutdown.as_str()),
            Command::Logout => self.spawn(self.config.logout.as_str()),
            Command::SpawnWiFiEditor => self.spawn(self.config.edit_wifi.as_str()),
            Command::SpawnBluetoothEditor => self.spawn(self.config.edit_bluetooth.as_str()),
            Command::SpawnSystemMonitor => self.spawn(self.config.open_system_monitor.as_str()),
            Command::ChangeWallpaper => self.spawn(self.config.change_wallpaper.as_str()),

            Command::TriggerTray { service, id } => {
                if let Some(tray) = &mut self.tray
                    && let Some(wants) = tray.wants_trigger(service, id)
                {
                    self.ring.schedule(ModuleId::Tray, wants);
                }
            }
        }

        self.ring.submit_if_dirty();
    }

    pub(crate) fn spawn(&self, cmd: &str) {
        SpawnHelper::spawn(cmd, self.home);
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
            && let Some(wants) = cpu.wants(&mut self.cpu_buf)
        {
            self.ring.schedule(ModuleId::Cpu, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(cpu) = &mut self.cpu {
            if cpu.satisfy(satisfy, &self.cpu_buf) == Ok(()) {
                IoSlice::<Cpu>::schedule(self);
            } else {
                log::error!("Stopping Cpu");
                self.cpu = None;
            }
        }
    }
}

impl IoSlice<Memory> for IO {
    fn schedule(&mut self) {
        if let Some(memory) = &mut self.memory
            && let Some(wants) = memory.wants(&mut self.memory_buf)
        {
            self.ring.schedule(ModuleId::Memory, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(memory) = &mut self.memory {
            if memory.satisfy(satisfy, &self.memory_buf) == Ok(()) {
                IoSlice::<Memory>::schedule(self);
            } else {
                log::error!("Stopping Memory");
                self.memory = None;
            }
        }
    }
}

impl IoSlice<KbMod> for IO {
    fn schedule(&mut self) {
        if let Some(kb_mod) = &mut self.kb_mod
            && let Some(wants) = kb_mod.wants(&self.kb_mod_addr, &mut self.kb_mod_buf)
        {
            self.ring.schedule(ModuleId::KbMod, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(kb_mod) = &mut self.kb_mod {
            if kb_mod.satisfy(satisfy, &mut self.kb_mod_buf) == Ok(()) {
                IoSlice::<KbMod>::schedule(self);
            } else {
                log::error!("Stopping KbMod");
                self.kb_mod = None;
            }
        }
    }
}

impl IoSlice<Weather> for IO {
    fn schedule(&mut self) {
        if let Some(weather) = &mut self.weather
            && let Some(wants) = weather.wants(&self.weather_addr, &mut self.weather_buf)
        {
            self.ring.schedule(ModuleId::Weather, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(weather) = &mut self.weather {
            if weather.satisfy(satisfy, &mut self.weather_buf) == Ok(()) {
                IoSlice::<Weather>::schedule(self);
            } else {
                log::error!("Stopping Weather");
                self.weather = None;
            }
        }
    }
}

impl IoSlice<NM> for IO {
    fn schedule(&mut self) {
        if let Some(nm) = &mut self.nm
            && let Some(wants) = nm.wants(&self.nm_addr, &mut self.nm_buf)
        {
            self.ring.schedule(ModuleId::NM, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(nm) = &mut self.nm {
            if nm.satisfy(satisfy, &mut self.nm_buf) == Ok(()) {
                IoSlice::<NM>::schedule(self);
            } else {
                log::error!("Stopping NM");
                self.nm = None;
            }
        }
    }
}

impl IoSlice<PW> for IO {
    fn schedule(&mut self) {
        if let Some(pw) = &mut self.pw
            && let Some(wants) = pw.wants(&self.pw_addr, &mut self.pw_buf)
        {
            self.ring.schedule(ModuleId::PW, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(pw) = &mut self.pw {
            if pw.satisfy(satisfy, &mut self.pw_buf) == Ok(()) {
                IoSlice::<PW>::schedule(self);
            } else {
                log::error!("Stopping PW");
                self.pw = None;
            }
        }
    }
}

impl IoSlice<Niri> for IO {
    fn schedule(&mut self) {
        if let Some(niri) = &mut self.niri
            && let Some(niri_addr) = &self.niri_addr
            && let Some(wants) = niri.wants(niri_addr, &mut self.niri_buf)
        {
            self.ring.schedule(ModuleId::Niri, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(niri) = &mut self.niri {
            if niri.satisfy(satisfy, &mut self.niri_buf) == Ok(()) {
                IoSlice::<Niri>::schedule(self);
            } else {
                log::error!("Stopping Niri");
                self.niri = None;
            }
        }
    }
}

impl IoSlice<Tray> for IO {
    fn schedule(&mut self) {
        if let Some(tray) = &mut self.tray
            && let Some(wants) = tray.wants(&self.tray_addr, &mut self.tray_buf)
        {
            self.ring.schedule(ModuleId::Tray, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(tray) = &mut self.tray {
            if tray.satisfy(satisfy, &mut self.tray_buf) == Ok(()) {
                IoSlice::<Tray>::schedule(self);
            } else {
                log::error!("Stopping Tray");
                self.tray = None;
            }
        }
    }
}

impl IoSlice<Control> for IO {
    fn schedule(&mut self) {
        if let Some(control) = &mut self.control {
            let wants = control.wants();
            self.ring.schedule(ModuleId::Control, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        if let Some(control) = &mut self.control {
            if control.satisfy(satisfy) == Ok(()) {
                IoSlice::<Control>::schedule(self);
            } else {
                log::error!("Stopping Control");
                self.control = None;
            }
        }
    }
}

impl IoSlice<TimerFd> for IO {
    fn schedule(&mut self) {
        if let Some(timer) = &mut self.timer
            && let Some(wants) = timer.wants(&mut self.timerbuf)
        {
            self.ring.schedule(ModuleId::Timer, wants);
        }
    }

    fn satisfy(&mut self, satisfy: Satisfy) {
        let Some(timer) = &mut self.timer else {
            return;
        };

        match timer.satisfy(satisfy, self.timerbuf) {
            Ok(Some(_tick)) => {
                IoSlice::<TimerFd>::schedule(self);

                self.clock.tick();

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
            Err(()) => {
                log::error!("Stopping Timer");
                self.timer = None;
            }
        }
    }
}
