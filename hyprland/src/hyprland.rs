use crate::{HyprlandEvent, reader::Reader, state::State, writer::Writer};
use anyhow::Result;
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Hyprland {
        #[pin]
        rx: UnboundedReceiver<HyprlandEvent>,
    }
}

const NAME: &str = "Hyprland";

impl Hyprland {
    pub fn new(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>, Hyprctl) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<HyprlandEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = Self::r#loop(tx, token).await {
                log::error!(target: "Hyprland", "{err:?}");
            }
        });
        (NAME, Self { rx }, handle, Hyprctl)
    }

    async fn r#loop(tx: UnboundedSender<HyprlandEvent>, token: CancellationToken) -> Result<()> {
        let mut reader = Reader::new().await?;

        let workspace_ids = Writer::get_workspaces_list().await?;
        let active_workspace_id = Writer::get_active_workspace().await?;
        let lang = Writer::get_language().await?;

        let mut state = State::new(workspace_ids, active_workspace_id, lang);

        for event in state.initial_events() {
            tx.send(event)?;
        }

        loop {
            tokio::select! {
                event = reader.next_event() => {
                    let event = event?;
                    let event = state.apply(event);
                    tx.send(event)?;
                },

                _ = token.cancelled() => {
                    log::info!(target: "Hyprland", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Stream for Hyprland {
    type Item = HyprlandEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}

pub struct Hyprctl;

impl Hyprctl {
    pub async fn dispatch(&self, cmd: impl AsRef<str>) {
        if let Err(err) = Writer::dispatch(cmd).await {
            log::error!(target: "Hyprland", "failed to dispatch hyprctl: {err:?}");
        }
    }
}
