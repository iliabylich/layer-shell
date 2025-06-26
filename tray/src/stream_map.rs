use crate::{dbus_event::DBusEvent, stream_id::StreamId};
use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;

pin_project! {
    pub(crate) struct StreamMap {
        #[pin]
        map: tokio_stream::StreamMap<
            StreamId,
            Pin<Box<dyn Stream<Item = DBusEvent> + Send + 'static>>,
        >,
        tx: UnboundedSender<DBusEvent>,
    }
}

impl StreamMap {
    pub(crate) fn new() -> Self {
        let mut map = tokio_stream::StreamMap::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<DBusEvent>();
        map.insert(StreamId::Manual, UnboundedReceiverStream::new(rx).boxed());

        Self { map, tx }
    }

    pub(crate) fn add<S>(&mut self, id: StreamId, stream: S)
    where
        S: Stream<Item = DBusEvent> + Send + 'static,
    {
        let stream = stream.boxed();
        self.map.insert(id, stream);
    }

    pub(crate) fn remove(&mut self, id: &StreamId) {
        self.map.remove(id);
    }

    pub(crate) fn remove_service(&mut self, service: &str) -> Option<usize> {
        let mut ids_to_remove = vec![];
        for id in self.map.keys() {
            if id.is_related_to(service) {
                ids_to_remove.push(id.clone())
            }
        }
        for id in ids_to_remove.iter() {
            self.remove(id);
        }
        let count = ids_to_remove.len();
        if count > 0 { Some(count) } else { None }
    }

    pub(crate) fn emit(&self, event: DBusEvent) -> Result<()> {
        self.tx
            .send(event)
            .context("failed to self-send message; closed stream")
    }
}

impl Stream for StreamMap {
    type Item = (StreamId, DBusEvent);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().map.poll_next(cx)
    }
}
