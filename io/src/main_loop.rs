use crate::{command::Command, event::Event};
use anyhow::{Result, anyhow, bail};
use clock::Clock;
use control::Control;
use cpu::CPU;
use futures::{Stream, StreamExt as _};
use hyprland::{Hyprland, hyprctl};
use memory::Memory;
use network::Network;
use std::{collections::HashMap, pin::Pin};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_stream::StreamMap;
use tokio_util::sync::CancellationToken;
use tray::Tray;
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
        register_task!(Tray);

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

    async fn emit(&self, module: &str, e: Event) -> Result<()> {
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }

    async fn on_command(&mut self, cmd: Command) {
        match cmd {
            Command::FinishIoThread => unreachable!("handled by the caller"),

            Command::HyprlandGoToWorkspace { idx } => {
                hyprctl!("workspace {}", idx + 1);
            }
            Command::Lock => {
                hyprctl!("exec hyprlock");
            }
            Command::Reboot => {
                hyprctl!("exec systemctl reboot");
            }
            Command::Shutdown => {
                hyprctl!("exec systemctl poweroff");
            }
            Command::Logout => {
                hyprctl!("exit");
            }
            Command::TriggerTray { uuid } => todo!(),
            Command::SpawnNetworkEditor => {
                hyprctl!("exec iwmenu --launcher fuzzel");
            }
            Command::SpawnSystemMonitor => {
                hyprctl!("exec gnome-system-monitor");
            }
            Command::ChangeTheme => {
                hyprctl!("exec ~/.config/hypr/wallpaper-change.sh");
            }
        }
    }
}
