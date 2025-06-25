use crate::{command::Command, event::Event};
use anyhow::{Result, anyhow, bail};
use clock::Clock;
use control::Control;
use cpu::CPU;
use futures::{Stream, StreamExt as _};
use hyprland::Hyprland;
use memory::Memory;
use network::Network;
use std::{collections::HashMap, pin::Pin};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_stream::StreamMap;
use tokio_util::sync::CancellationToken;
use weather::Weather;

pub(crate) struct MainLoop {
    token: CancellationToken,
    handles: HashMap<&'static str, JoinHandle<()>>,

    etx: UnboundedSender<Event>,
    crx: UnboundedReceiver<Command>,

    streams: StreamMap<&'static str, Pin<Box<dyn Stream<Item = Event> + Send + 'static>>>,
}

impl MainLoop {
    pub(crate) async fn new(
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
            ($t:ty) => {
                let (name, stream, handle) = <$t>::new(token.clone());
                handles.insert(name, handle);
                streams.insert(name, stream.map(Event::from).boxed());
            };
        }

        register_stream!(Clock);
        register_stream!(CPU);
        register_stream!(Memory);
        register_task!(Control);
        register_task!(Hyprland);
        register_task!(Network);
        register_task!(Weather);

        Ok(Self {
            token,
            handles,

            etx,
            crx,

            streams,
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

    async fn emit(&self, module: &str, e: impl Into<Event>) -> Result<()> {
        let e: Event = e.into();
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }

    async fn hyprctl_dispatch(&self, cmd: impl AsRef<str>) {
        if let Err(err) = Hyprland::hyprctl_dispatch(cmd).await {
            log::error!("{err:?}");
        }
    }

    async fn on_command(&mut self, cmd: Command) {
        match cmd {
            Command::FinishIoThread => unreachable!("handled by the caller"),

            Command::HyprlandGoToWorkspace { idx } => {
                self.hyprctl_dispatch(format!("workspace {}", idx + 1))
                    .await;
            }
            Command::Lock => {
                self.hyprctl_dispatch("exec hyprlock").await;
            }
            Command::Reboot => {
                self.hyprctl_dispatch("exec systemctl reboot").await;
            }
            Command::Shutdown => {
                self.hyprctl_dispatch("exec systemctl poweroff").await;
            }
            Command::Logout => {
                self.hyprctl_dispatch("exit").await;
            }
            Command::TriggerTray { uuid } => todo!(),
            Command::SpawnNetworkEditor => {
                self.hyprctl_dispatch("exec iwmenu --launcher fuzzel").await;
            }
            Command::SpawnSystemMonitor => {
                self.hyprctl_dispatch("exec gnome-system-monitor").await;
            }
            Command::ChangeTheme => {
                self.hyprctl_dispatch("exec ~/.config/hypr/wallpaper-change.sh")
                    .await
            }
        }
    }
}
