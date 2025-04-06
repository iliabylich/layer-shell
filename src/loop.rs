use crate::{
    Command, Event, VerboseSender,
    channel::SignalingCommandReceiver,
    fatal,
    fd_id::FdId,
    modules::{
        Module,
        control::Control,
        cpu::CPU,
        hyprland::Hyprland,
        launcher::{GlobalLauncherWatcher, Launcher, UserLauncherWatcher},
        memory::Memory,
        network::Network,
        pipewire::Pipewire,
        session::Session,
        time::Time,
        tray::Tray,
        weather::Weather,
    },
    timer::Timer,
};
use anyhow::{Context as _, Result};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use std::os::fd::{AsRawFd, RawFd};

pub(crate) struct Loop {
    poll: Poll,

    timer: Option<Timer>,

    time: Time,
    memory: Memory,
    cpu: CPU,
    hyprland: Option<Hyprland>,
    control: Option<Control>,
    pipewire: Option<Pipewire>,
    network: Option<Network>,
    launcher: Launcher,
    global_launcher_watcher: Option<GlobalLauncherWatcher>,
    user_launcher_watcher: Option<UserLauncherWatcher>,
    tray: Option<Tray>,
    tx: VerboseSender<Event>,
    rx: SignalingCommandReceiver,
    weather: Option<Weather>,
}

impl Loop {
    pub(crate) fn new(tx: VerboseSender<Event>, rx: SignalingCommandReceiver) -> Result<Self> {
        let poll = Poll::new()?;

        let timer = make_module_with_fd_id::<Timer>(&tx, &poll);
        let time = Time::new(tx.clone());
        let memory = Memory::new(tx.clone());
        let cpu = CPU::new(tx.clone());
        let hyprland = make_module_with_fd_id::<Hyprland>(&tx, &poll);
        let control = make_module_with_fd_id::<Control>(&tx, &poll);
        let pipewire = make_module_with_fd_id::<Pipewire>(&tx, &poll);
        let network = make_module_with_fd_id::<Network>(&tx, &poll);

        let launcher = Launcher::new(&tx);
        let mut global_launcher_watcher =
            make_module_with_fd_id::<GlobalLauncherWatcher>(&tx, &poll);
        if let Some(watcher) = global_launcher_watcher.as_mut() {
            watcher.connect(&launcher);
        }
        let mut user_launcher_watcher = make_module_with_fd_id::<UserLauncherWatcher>(&tx, &poll);
        if let Some(watcher) = user_launcher_watcher.as_mut() {
            watcher.connect(&launcher);
        }

        let tray = make_module_with_fd_id::<Tray>(&tx, &poll);

        register_reader(&poll, rx.as_raw_fd(), SignalingCommandReceiver::TOKEN)?;

        let this = Self {
            poll,

            timer,

            time,
            memory,
            cpu,
            hyprland,
            control,
            pipewire,
            network,
            launcher,
            global_launcher_watcher,
            user_launcher_watcher,
            tray,
            tx,
            rx,
            weather: None,
        };

        Ok(this)
    }

    fn tick(&mut self, events: &mut Events) {
        if let Err(err) = poll(&mut self.poll, events) {
            log::error!("{err:?}");
            return;
        }

        for event in events.iter() {
            match event.token() {
                Timer::TOKEN => {
                    if let Some(ticks) = read_events_or_disable(&mut self.timer, &self.poll) {
                        if ticks.is_multiple_of(Time::INTERVAL) {
                            self.time.tick();
                        }
                        if ticks.is_multiple_of(Memory::INTERVAL) {
                            self.memory.tick();
                        }
                        if ticks.is_multiple_of(CPU::INTERVAL) {
                            self.cpu.tick();
                        }
                        if ticks.is_multiple_of(Weather::INTERVAL) {
                            self.weather = make_module_with_fd_id::<Weather>(&self.tx, &self.poll);
                        }
                    }
                }

                Hyprland::TOKEN => {
                    read_events_or_disable(&mut self.hyprland, &self.poll);
                }

                Control::TOKEN => {
                    read_events_or_disable(&mut self.control, &self.poll);
                }

                Pipewire::TOKEN => {
                    read_events_or_disable(&mut self.pipewire, &self.poll);
                }

                Network::TOKEN => {
                    read_events_or_disable(&mut self.network, &self.poll);
                }

                GlobalLauncherWatcher::TOKEN => {
                    read_events_or_disable(&mut self.global_launcher_watcher, &self.poll);
                }

                UserLauncherWatcher::TOKEN => {
                    read_events_or_disable(&mut self.user_launcher_watcher, &self.poll);
                }

                Tray::TOKEN => {
                    read_events_or_disable(&mut self.tray, &self.poll);
                }

                Weather::TOKEN => {
                    read_events_or_disable(&mut self.weather, &self.poll);
                }

                SignalingCommandReceiver::TOKEN => {
                    self.rx.consume_signal();
                    while let Some(cmd) = self.rx.recv() {
                        if let Err(err) = self.process_command(cmd) {
                            log::error!("failed to process command: {err:?}");
                        }
                    }
                }

                _ => unreachable!(),
            }
        }
    }

    pub(crate) fn start(mut self) -> ! {
        let mut events = Events::with_capacity(100);

        loop {
            self.tick(&mut events);
        }
    }

    fn process_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::HyprlandGoToWorkspace { idx } => Hyprland::go_to_workspace(idx)?,
            Command::LauncherReset => self.launcher.reset(),
            Command::LauncherGoUp => self.launcher.go_up(),
            Command::LauncherGoDown => self.launcher.go_down(),
            Command::LauncherSetSearch { search } => self.launcher.set_search(search),
            Command::LauncherExecSelected => self.launcher.exec_selected(),
            Command::Lock => Session::lock(),
            Command::Reboot => Session::reboot(),
            Command::Shutdown => Session::shutdown(),
            Command::Logout => Session::logout(),
            Command::ChangeTheme => Session::change_theme(),
            Command::TriggerTray { uuid } => {
                self.tray
                    .as_mut()
                    .context("tray module is not running")?
                    .trigger(uuid)?;
            }
            Command::SpawnNetworkEditor => Network::spawn_network_editor()?,
            Command::SpawnSystemMonitor => Memory::spawn_system_monitor(),
        }

        Ok(())
    }
}

fn register_reader(poll: &Poll, fd: RawFd, token: Token) -> Result<()> {
    let fd_id = FdId::from(token);
    poll.registry()
        .register(&mut SourceFd(&fd), token, Interest::READABLE)
        .with_context(|| format!("failed to register {fd} with {token:?} ({fd_id:?}) in epoll"))?;
    log::info!("[epoll] registered fd {fd} with token {token:?} ({fd_id:?})");
    Ok(())
}

fn unregister_reader(poll: &Poll, fd: RawFd) {
    if let Err(err) = poll.registry().deregister(&mut SourceFd(&fd)) {
        log::error!("[epoll] failed to un-register {fd} from epoll: {err:?}");
    }
}

fn poll(poll: &mut Poll, events: &mut Events) -> Result<()> {
    poll.poll(events, None).context("failed to poll epoll")?;
    Ok(())
}

fn make_module_with_fd_id<T>(tx: &VerboseSender<Event>, poll: &Poll) -> Option<T>
where
    T: Module,
{
    match T::new(tx.clone()) {
        Ok(module) => {
            if let Err(err) = register_reader(poll, module.as_raw_fd(), T::FD_ID.token()) {
                fatal!("[epoll] failed to register reader {:?}: {err:?}", T::FD_ID);
            }
            Some(module)
        }
        Err(err) => {
            log::error!("[{}] {err:?}", T::NAME);
            None
        }
    }
}

fn read_events_or_disable<T>(module: &mut Option<T>, poll: &Poll) -> Option<T::ReadOutput>
where
    T: Module,
{
    match module.as_mut() {
        Some(reader) => match reader.read_events() {
            Ok(output) => Some(output),
            Err(err) => {
                log::error!("[{}] {err:?}", T::NAME);
                unregister_reader(poll, reader.as_raw_fd());
                None
            }
        },
        None => {
            log::error!("[{}] unexpected epoll event", T::NAME);
            None
        }
    }
}
