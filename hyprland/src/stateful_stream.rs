use crate::{raw_stream::RawStream, state::State, Event};
use anyhow::Result;
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) struct StatefulStream {
    inner: RawStream,

    state: State,

    emitted_start_workspaces_event: bool,
    emitted_start_language_event: bool,
}

impl StatefulStream {
    pub(crate) async fn new() -> Result<Self> {
        Ok(Self {
            inner: RawStream::new().await?,
            state: State::new().await?,
            emitted_start_workspaces_event: false,
            emitted_start_language_event: false,
        })
    }
}

impl Stream for StatefulStream {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.emitted_start_workspaces_event {
            self.emitted_start_workspaces_event = true;
            return Poll::Ready(Some(self.state.as_workspaces_changed_event()));
        }

        if !self.emitted_start_language_event {
            self.emitted_start_language_event = true;
            return Poll::Ready(Some(self.state.as_language_changed_event()));
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(self.state.apply(event))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
