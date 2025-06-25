use crate::{Event, reader::Reader, state::State, writer::Writer};
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
        rx: UnboundedReceiver<Event>,
        #[pin]
        handle: JoinHandle<()>,
        token: CancellationToken
    }
}

impl Hyprland {
    pub fn new(token: CancellationToken) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let handle = {
            let token = token.clone();
            tokio::task::spawn(async move {
                if let Err(err) = Self::r#loop(tx, token).await {
                    log::error!("Hyprland crashed: {err:?}");
                }
            })
        };
        Self { rx, handle, token }
    }

    async fn r#loop(tx: UnboundedSender<Event>, token: CancellationToken) -> Result<()> {
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

    pub async fn hyprctl_dispatch(&self, cmd: impl AsRef<str>) -> Result<()> {
        Writer::dispatch(cmd).await
    }
}

impl Stream for Hyprland {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}

impl Future for Hyprland {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = futures::ready!(self.project().handle.poll(cx)) {
            log::error!("failed to await Hyprland task: {err:?}")
        }
        std::task::Poll::Ready(())
    }
}
