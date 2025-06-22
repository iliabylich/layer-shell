use crate::{command::Command, event::Event};
use anyhow::{Context as _, Result, anyhow, bail};
use futures_util::{StreamExt as _, stream::Fuse};
use hyprland::Hyprland;
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) struct MainLoop {
    etx: Sender<Event>,
    crx: Receiver<Command>,

    hyprland: Fuse<Hyprland>,
}

impl MainLoop {
    pub(crate) async fn new(etx: Sender<Event>, crx: Receiver<Command>) -> Result<Self> {
        let hyprland = Hyprland::new().await?.fuse();

        Ok(Self { etx, crx, hyprland })
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        loop {
            self.tick().await?;
        }
    }

    async fn tick(&mut self) -> Result<()> {
        tokio::select! {
            Some(e) = self.hyprland.next() => {
                self.emit("Hyprland", e).await?;
            }

            else => bail!("all streams are dead"),
        }

        Ok(())
    }

    async fn emit(&self, module: &str, e: impl Into<Event>) -> Result<()> {
        let e: Event = e.into();
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .await
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }
}
