use crate::{
    Event,
    reader::{Reader, ReaderEvent},
    state::State,
    writer::Writer,
};
use anyhow::Result;
use utils::{Emitter, service};

struct Task {
    emitter: Emitter<Event>,
    exit: tokio::sync::oneshot::Receiver<()>,
    reader: Reader,
    state: State,
}

impl Task {
    async fn start(
        emitter: Emitter<Event>,
        exit: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        let reader = Reader::new().await?;

        let workspace_ids = Writer::get_workspaces_list().await?;
        let active_workspace_id = Writer::get_active_workspace().await?;
        let lang = Writer::get_language().await?;

        let (state, events) = State::new(workspace_ids, active_workspace_id, lang);

        for event in events {
            emitter.emit(event).await?;
        }

        Self {
            emitter,
            exit,
            reader,
            state,
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                event = self.reader.next_event() => self.on_event(event?).await?,

                _ = &mut self.exit => {
                    log::info!(target: "Hyprland", "exiting...");
                    return Ok(())
                }
            }
        }
    }

    async fn on_event(&mut self, event: ReaderEvent) -> Result<()> {
        let event = match event {
            ReaderEvent::CreateWorkspace(id) => self.state.add_workspace(id),
            ReaderEvent::DestroyWorkspace(id) => self.state.remove_workspace(id),
            ReaderEvent::Workspace(id) => self.state.set_active_workspace(id),
            ReaderEvent::LanguageChanged(lang) => self.state.set_language(lang),
        };
        self.emitter.emit(event).await
    }
}

service!(Hyprland, Event, Task::start);

impl Hyprland {
    pub async fn hyprctl_dispatch(&mut self, cmd: impl AsRef<str>) -> Result<()> {
        Writer::dispatch(cmd).await
    }
}
