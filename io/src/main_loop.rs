use crate::{command::Command, config::Config, event::Event};
use anyhow::{Result, anyhow, bail};
use clock::Clock;
use control::Control;
use cpu::CPU;
use futures::{Stream, StreamExt as _};
use hyprland::{Hyprctl, Hyprland};
use memory::Memory;
use network::Network;
use std::{collections::HashMap, pin::Pin};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_stream::StreamMap;
use tokio_util::sync::CancellationToken;
use tray::{Tray, TrayCtl};
use weather::Weather;

pub(crate) struct MainLoop {
    config: Config,
    token: CancellationToken,
    handles: HashMap<&'static str, JoinHandle<()>>,

    etx: UnboundedSender<Event>,
    crx: UnboundedReceiver<Command>,

    streams: StreamMap<&'static str, Pin<Box<dyn Stream<Item = Event> + Send + 'static>>>,

    hyprctl: Hyprctl,
    trayctl: TrayCtl,
}

impl MainLoop {
    pub(crate) async fn new(
        config: Config,
        etx: UnboundedSender<Event>,
        crx: UnboundedReceiver<Command>,
    ) -> Result<Self> {
        let token = CancellationToken::new();

        let mut handles = HashMap::new();
        let mut streams = StreamMap::new();

        macro_rules! register_stream {
            ($t:ty) => {{
                let (name, stream) = <$t>::new();
                streams.insert(name, stream.map(Event::from).boxed());
            }};
        }
        macro_rules! register_task {
            ($t:ty) => {{
                let (name, stream, handle, out) = <$t>::new(token.clone());
                handles.insert(name, handle);
                streams.insert(name, stream.map(Event::from).boxed());
                out
            }};
        }

        register_stream!(Clock);
        register_stream!(CPU);
        register_stream!(Memory);
        register_task!(Control);
        register_task!(Network);
        register_task!(Weather);
        let hyprctl = register_task!(Hyprland);
        let trayctl = register_task!(Tray);

        Ok(Self {
            config,
            token,
            handles,

            etx,
            crx,

            streams,

            hyprctl,
            trayctl,
        })
    }

    pub(crate) async fn start(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some((name, event)) = self.streams.next() => {
                    self.emit(name, event).await?;
                }

                Some(cmd) = self.crx.recv() => {
                    if matches!(cmd, Command::FinishIoThread) {
                        self.stop().await;
                        return Ok(());
                    }
                    self.on_command(cmd).await;
                }

                else => bail!("all streams are dead"),
            }
        }
    }

    async fn stop(self) {
        self.token.cancel();

        for (name, handle) in self.handles {
            if let Err(err) = handle.await {
                log::error!("failed to await for {name} completion: {err:?}");
            }
        }
    }

    async fn emit(&self, module: &str, e: Event) -> Result<()> {
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }

    async fn on_command(&mut self, cmd: Command) {
        macro_rules! hyprctl {
            ($($arg:tt)*) => {
                self.hyprctl.dispatch(format!($($arg)*)).await
            };
        }

        match cmd {
            Command::FinishIoThread => unreachable!("handled by the caller"),

            Command::HyprlandGoToWorkspace { workspace } => {
                hyprctl!("workspace {}", workspace);
            }
            Command::Lock => {
                hyprctl!("exec {}", self.config.lock);
            }
            Command::Reboot => {
                hyprctl!("exec {}", self.config.reboot);
            }
            Command::Shutdown => {
                hyprctl!("exec {}", self.config.shutdown);
            }
            Command::Logout => {
                hyprctl!("exit");
            }
            Command::TriggerTray { uuid } => {
                self.trayctl.trigger(uuid);
            }
            Command::SpawnWiFiEditor => {
                hyprctl!("exec {}", self.config.edit_wifi);
            }
            Command::SpawnBluetoothEditor => {
                hyprctl!("exec {}", self.config.edit_bluetooth);
            }
            Command::SpawnSystemMonitor => {
                hyprctl!("exec {}", self.config.open_system_monitor);
            }
            Command::ChangeTheme => {
                hyprctl!("exec {}", self.config.change_theme);
            }
        }
    }
}
