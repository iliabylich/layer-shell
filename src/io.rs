use crate::{
    IoEvent,
    command::Command,
    config::Config,
    emitter::Emitter,
    module_id::ModuleId,
    modules::{
        Clock, Control, Cpu, KbMod, Memory, Module, NM, Niri, OptionModuleExt, PW, Timer, Tray,
        Weather,
    },
    utils::{EnvHelper, Epoll, ModulePoll, SpawnHelper},
};
use rustix::fd::{AsFd, AsRawFd};

pub struct IO {
    epoll: Epoll,
    emitter: Emitter,

    pub(crate) config: Config,

    timer: Option<Timer>,

    pw: Option<PW>,
    nm: Option<NM>,
    weather: Option<Weather>,
    cpu: Option<Cpu>,
    memory: Option<Memory>,
    kb_mod: Option<KbMod>,
    niri: Option<Niri>,
    tray: Option<Tray>,
    control: Option<Control>,
    running: bool,

    home: &'static str,
}

impl IO {
    pub(crate) fn new(
        callback: extern "C" fn(event: &IoEvent, *mut core::ffi::c_void),
        data: *mut core::ffi::c_void,
    ) -> Self {
        log::trace!("Creating IO");

        let home = EnvHelper::home();
        let xdg_runtime_dir = EnvHelper::xdg_runtime_dir();
        let xdg_config_dir = EnvHelper::xdg_config_dir();

        Self {
            epoll: Epoll::new(),
            emitter: Emitter::new(callback, data),

            config: Config::read(xdg_config_dir, home),

            timer: Timer::new(),
            pw: PW::new(xdg_runtime_dir),
            nm: NM::new(),
            weather: Weather::new(xdg_runtime_dir),
            cpu: Cpu::new(),
            memory: Memory::new(),
            kb_mod: KbMod::new(),
            niri: Niri::new(),
            tray: Tray::new(xdg_runtime_dir),
            control: Control::new(xdg_runtime_dir),
            running: true,

            home,
        }
    }

    pub(crate) fn start(&self) {
        macro_rules! start {
            ($var:expr) => {
                if let Some(inner) = &$var {
                    self.epoll.add(inner, inner.id());
                }
            };
        }
        start!(self.timer);
        start!(self.pw);
        start!(self.nm);
        start!(self.kb_mod);
        start!(self.weather);
        start!(self.niri);
        start!(self.tray);
        start!(self.control);
    }

    pub(crate) fn handle_readable(&mut self) {
        if !self.running {
            return;
        }
        for module_poll in self.epoll.wait() {
            match module_poll {
                ModulePoll::Err(module_id) => self.stop_module(module_id),
                ModulePoll::Readable(module_id) => self.read_module(module_id),
                ModulePoll::None => {}
            }
        }
    }

    pub(crate) fn wait_readable(&mut self) {
        self.epoll.wait_readable();
    }

    pub(crate) const fn stop(&mut self) {
        self.running = false;
    }

    fn stop_module(&mut self, module_id: ModuleId) {
        log::error!("Stopping {module_id:?}");

        macro_rules! stop {
            ($var:expr) => {
                if let Some(inner) = &$var {
                    self.epoll.delete(inner);
                    $var = None
                }
            };
        }

        match module_id {
            ModuleId::KbMod => stop!(self.kb_mod),
            ModuleId::NM => stop!(self.nm),
            ModuleId::PW => stop!(self.pw),
            ModuleId::Niri => stop!(self.niri),
            ModuleId::Weather => stop!(self.weather),
            ModuleId::Tray => stop!(self.tray),
            ModuleId::Control => stop!(self.control),
            ModuleId::Timer => stop!(self.timer),
            ModuleId::Cpu => self.cpu = None,
            ModuleId::Memory => self.memory = None,
        }
    }

    fn read_module(&mut self, module_id: ModuleId) {
        macro_rules! read {
            ($var:expr) => {{
                let id = $var.id();
                let inner = $var
                    .as_mut()
                    .unwrap_or_else(|| panic!("{id:?} is readable but not set"));
                if inner.read(self.emitter).is_err() {
                    self.stop_module(id);
                    return;
                }
            }};
        }

        match module_id {
            ModuleId::KbMod => read!(self.kb_mod),
            ModuleId::NM => read!(self.nm),
            ModuleId::PW => read!(self.pw),
            ModuleId::Niri => read!(self.niri),
            ModuleId::Weather => read!(self.weather),
            ModuleId::Tray => read!(self.tray),
            ModuleId::Control => read!(self.control),
            ModuleId::Cpu => {
                if self.cpu.is_some() {
                    read!(self.cpu);
                }
            }
            ModuleId::Memory => {
                if self.memory.is_some() {
                    read!(self.memory);
                }
            }
            ModuleId::Timer => {
                read!(self.timer);
                self.read_module(ModuleId::Cpu);
                self.read_module(ModuleId::Memory);
                Clock::tick(self.emitter);
            }
        }
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
                if let Some(tray) = &mut self.tray {
                    tray.trigger(service, id);
                }
            }
        }
    }

    pub(crate) fn spawn(&self, cmd: &str) {
        SpawnHelper::spawn(cmd, self.home);
    }

    pub(crate) fn fd(&self) -> i32 {
        self.epoll.as_fd().as_raw_fd()
    }
}
