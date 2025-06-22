use crate::{
    Event,
    reader::{Reader, ReaderEvent},
    state::State,
    writer::Writer,
};
use anyhow::Result;
use futures_util::{Stream, ready};
use pin_project_lite::pin_project;
use std::task::Poll::*;

pin_project! {
    pub struct HyprlandStream {
        #[pin]
        reader: Reader,

        buf: Vec<Event>,
        state: State,
    }
}

impl HyprlandStream {
    pub async fn new() -> Result<Self> {
        let reader = Reader::new().await?;

        let workspace_ids = Writer::get_workspaces_list().await?;
        let active_workspace_id = Writer::get_active_workspace().await?;
        let lang = Writer::get_language().await?;

        let (state, buf) = State::new(workspace_ids, active_workspace_id, lang);

        Ok(Self { reader, buf, state })
    }

    pub async fn hyprctl_dispatch(&mut self, cmd: impl AsRef<str>) -> Result<()> {
        Writer::dispatch(cmd).await
    }
}

impl Stream for HyprlandStream {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        if let Some(event) = this.buf.pop() {
            return Ready(Some(event));
        }

        let Some(event) = ready!(this.reader.as_mut().poll_next(cx)) else {
            return Ready(None);
        };

        let event = match event {
            ReaderEvent::CreateWorkspace(id) => this.state.add_workspace(id),
            ReaderEvent::DestroyWorkspace(id) => this.state.remove_workspace(id),
            ReaderEvent::Workspace(id) => this.state.set_active_workspace(id),
            ReaderEvent::LanguageChanged(lang) => this.state.set_language(lang),
        };

        Ready(Some(event))
    }
}
