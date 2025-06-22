use crate::{
    Event,
    reader::{Reader, ReaderEvent},
    state::State,
    writer::Writer,
};
use anyhow::{Context as _, Result};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub(crate) struct Task {
    tx: Sender<Event>,
}

impl Task {
    pub(crate) fn spawn() -> (Receiver<Event>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        let handle = tokio::spawn(async move {
            let task = Self { tx };
            if let Err(err) = task.start().await {
                log::error!("{err:?}");
            }
        });

        (rx, handle)
    }

    async fn start(self) -> Result<()> {
        let mut reader = Reader::new().await?;

        let workspace_ids = Writer::get_workspaces_list().await?;
        let active_workspace_id = Writer::get_active_workspace().await?;
        let lang = Writer::get_language().await?;

        let (mut state, events) = State::new(workspace_ids, active_workspace_id, lang);

        for event in events {
            self.send(event).await?;
        }

        loop {
            let event = reader.next_event().await?;
            let event = match event {
                ReaderEvent::CreateWorkspace(id) => state.add_workspace(id),
                ReaderEvent::DestroyWorkspace(id) => state.remove_workspace(id),
                ReaderEvent::Workspace(id) => state.set_active_workspace(id),
                ReaderEvent::LanguageChanged(lang) => state.set_language(lang),
            };
            self.send(event).await?;
        }
    }

    async fn send(&self, event: Event) -> Result<()> {
        self.tx
            .send(event)
            .await
            .context("failed to send event, channel is closed")
    }
}
