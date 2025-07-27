use crate::{Hyprctl, HyprlandEvent, reader::Reader, state::State, writer::Writer};
use anyhow::Result;
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Hyprland {
    etx: UnboundedSender<HyprlandEvent>,
    crx: UnboundedReceiver<String>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Hyprland {
    const NAME: &str = "Hyprland";

    type Event = HyprlandEvent;
    type Command = String;

    type Ctl = Hyprctl;

    fn new(
        etx: UnboundedSender<Self::Event>,
        crx: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
        _: TimerSubscriber,
    ) -> Self {
        Self { etx, crx, token }
    }

    async fn start(&mut self) -> Result<()> {
        let mut reader = Reader::new().await?;

        let workspace_ids = Writer::get_workspaces_list().await?;
        let active_workspace_id = Writer::get_active_workspace().await?;
        let lang = Writer::get_language().await?;

        let mut state = State::new(workspace_ids, active_workspace_id, lang);

        for event in state.initial_events() {
            self.etx.send(event)?;
        }

        loop {
            tokio::select! {
                event = reader.next_event() => {
                    let event = event?;
                    let event = state.apply(event);
                    self.etx.send(event)?;
                },

                Some(command) = self.crx.recv() => {
                    exec_in_place(command).await;
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Hyprland", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

async fn exec_in_place(command: String) {
    if let Err(err) = Writer::dispatch(command).await {
        log::error!(target: "Hyprland", "failed to dispatch hyprctl: {err:?}");
    }
}
