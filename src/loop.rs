use std::ops::ControlFlow;

use crate::{
    Command,
    channel::{CommandReceiver, EventSender},
    modules::{
        MaybeModule, MaybeTickingModule, Module,
        clock::Clock,
        control::Control,
        cpu::CPU,
        hyprland::Hyprland,
        launcher::{GlobalLauncherWatcher, Launcher, UserLauncherWatcher},
        memory::Memory,
        network::Network,
        pipewire::Pipewire,
        session::Session,
        tray::Tray,
        weather::Weather,
    },
    poll::Poll,
    timer::Timer,
};
use anyhow::{Context as _, Result};
use mio::Events;

pub(crate) struct Loop {
    poll: Poll,

    clock: Option<Clock>,
    memory: Option<Memory>,
    cpu: Option<CPU>,
    weather: Option<Weather>,
    launcher: Launcher,

    timer: Option<Timer>,
    hyprland: Option<Hyprland>,
    control: Option<Control>,
    pipewire: Option<Pipewire>,
    network: Option<Network>,
    global_launcher_watcher: Option<GlobalLauncherWatcher>,
    user_launcher_watcher: Option<UserLauncherWatcher>,
    tray: Option<Tray>,

    tx: EventSender,
    rx: CommandReceiver,
}

impl Loop {
    pub(crate) fn new(tx: EventSender, rx: CommandReceiver) -> Result<Self> {
        let poll = Poll::new()?;

        let clock = Some(Clock::new(&tx));
        let memory = Some(Memory::new(&tx));
        let cpu = Some(CPU::new(&tx));
        let weather = None;
        let launcher = Launcher::new(&tx);

        let timer = Timer::try_new(&tx);
        let hyprland = Hyprland::try_new(&tx);
        let control = Control::try_new(&tx);
        let pipewire = Pipewire::try_new(&tx);
        let network = Network::try_new(&tx);
        let mut global_launcher_watcher = GlobalLauncherWatcher::try_new(&tx);
        let mut user_launcher_watcher = UserLauncherWatcher::try_new(&tx);
        let tray = Tray::try_new(&tx);

        poll.add_maybe_reader(&timer);
        poll.add_maybe_reader(&hyprland);
        poll.add_maybe_reader(&control);
        poll.add_maybe_reader(&pipewire);
        poll.add_maybe_reader(&network);
        poll.add_maybe_reader(&global_launcher_watcher);
        poll.add_maybe_reader(&user_launcher_watcher);
        poll.add_maybe_reader(&tray);
        poll.add_reader(&rx);

        if let Some(watcher) = global_launcher_watcher.as_mut() {
            watcher.connect(&launcher);
        }
        if let Some(watcher) = user_launcher_watcher.as_mut() {
            watcher.connect(&launcher);
        }

        Ok(Self {
            poll,

            clock,
            memory,
            cpu,
            weather,
            launcher,

            timer,
            hyprland,
            control,
            pipewire,
            network,
            global_launcher_watcher,
            user_launcher_watcher,
            tray,

            tx,
            rx,
        })
    }

    fn tick(&mut self, events: &mut Events) -> ControlFlow<()> {
        if let Err(err) = self.poll.poll(events) {
            log::error!("{err:?}");
            return ControlFlow::Continue(());
        }

        for event in events.iter() {
            match event.token() {
                Timer::TOKEN => {
                    if let Some(ticks) = self.timer.read_events_or_unregister(&self.poll) {
                        if ticks.is_multiple_of(Clock::INTERVAL) {
                            self.clock.tick();
                        }
                        if ticks.is_multiple_of(Memory::INTERVAL) {
                            self.memory.tick();
                        }
                        if ticks.is_multiple_of(CPU::INTERVAL) {
                            self.cpu.tick();
                        }
                        if ticks.is_multiple_of(Weather::INTERVAL) {
                            self.weather = Weather::try_new(&self.tx);
                            self.poll.add_maybe_reader(&self.weather);
                        }
                    }
                }

                Hyprland::TOKEN => {
                    self.hyprland.read_events_or_unregister(&self.poll);
                }

                Control::TOKEN => {
                    self.control.read_events_or_unregister(&self.poll);
                }

                Pipewire::TOKEN => {
                    self.pipewire.read_events_or_unregister(&self.poll);
                }

                Network::TOKEN => {
                    self.network.read_events_or_unregister(&self.poll);
                }

                GlobalLauncherWatcher::TOKEN => {
                    self.global_launcher_watcher
                        .read_events_or_unregister(&self.poll);
                }

                UserLauncherWatcher::TOKEN => {
                    self.user_launcher_watcher
                        .read_events_or_unregister(&self.poll);
                }

                Tray::TOKEN => {
                    self.tray.read_events_or_unregister(&self.poll);
                }

                Weather::TOKEN => {
                    self.weather.read_events_or_unregister(&self.poll);
                    self.weather = None;
                }

                CommandReceiver::TOKEN => {
                    self.rx.consume_signal();
                    while let Some(cmd) = self.rx.recv() {
                        if let Command::FinishIoThread = cmd {
                            return ControlFlow::Break(());
                        }
                        if let Err(err) = self.process_command(cmd) {
                            log::error!("failed to process command: {err:?}");
                        }
                    }
                }

                _ => unreachable!(),
            }
        }

        ControlFlow::Continue(())
    }

    pub(crate) fn start(mut self) {
        let mut events = Events::with_capacity(100);

        loop {
            if let ControlFlow::Break(_) = self.tick(&mut events) {
                break;
            }
        }
    }

    fn process_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::FinishIoThread => unreachable!("FinishIoThread is processed by the caller"),
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
