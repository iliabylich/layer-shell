use crate::{command::Command, config::Config, event::Event};
use anyhow::{Context as _, Result, anyhow};
use clock::Clock;
use control::Control;
use cpu::CPU;
use futures::{StreamExt as _, stream::BoxStream};
use hyprland::{Hyprctl, Hyprland};
use memory::Memory;
use module::{Ctl, Module, Timer};
use network::Network;
use sound::Sound;
use std::{
    collections::HashMap,
    io::{PipeWriter, Write},
};
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

    timer: Timer,
    streams: StreamMap<&'static str, BoxStream<'static, Event>>,

    hyprctl: Hyprctl,
    trayctl: TrayCtl,

    pipe_writer: PipeWriter,
}

impl MainLoop {
    pub(crate) async fn new(
        config: Config,
        etx: UnboundedSender<Event>,
        crx: UnboundedReceiver<Command>,
        pipe_writer: PipeWriter,
    ) -> Result<Self> {
        let token = CancellationToken::new();

        let mut handles = HashMap::new();
        let mut streams = StreamMap::new();

        let timer = Timer::new();

        macro_rules! register {
            ($t:ty) => {{
                let (stream, handle, ctl) = <$t>::spawn(token.clone(), timer.subscribe());
                handles.insert(<$t>::NAME, handle);
                streams.insert(<$t>::NAME, stream.map(Event::from).boxed());
                ctl
            }};
        }

        register!(Memory);
        register!(CPU);
        register!(Clock);
        register!(Control);
        register!(Network);
        register!(Weather);
        register!(Sound);
        let hyprctl = register!(Hyprland);
        let trayctl = register!(Tray);

        Ok(Self {
            config,
            token,
            handles,

            etx,
            crx,

            timer,
            streams,

            hyprctl,
            trayctl,

            pipe_writer,
        })
    }

    pub(crate) async fn start(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = &mut self.timer => {
                    self.timer.tick()?;
                }

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
            }
        }
    }

    async fn stop(self) {
        self.token.cancel();

        for (name, handle) in self.handles {
            if let Err(err) = handle.await {
                log::error!(target: name, "failed to await for completion: {err:?}");
            }
        }
    }

    async fn emit(&mut self, module: &str, e: Event) -> Result<()> {
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))?;

        self.pipe_writer
            .write(b"1")
            .context("failed to write to pipe")?;

        Ok(())
    }

    async fn on_command(&mut self, cmd: Command) {
        macro_rules! hyprctl {
            ($($arg:tt)*) => {
                self.hyprctl.send(format!($($arg)*)).await
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
                self.trayctl.send(uuid).await;
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
