use crate::nm_event::NetworkManagerEvent;
use anyhow::{Context as _, Result};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;

type InnerMap = tokio_stream::StreamMap<
    &'static str,
    Pin<Box<dyn Stream<Item = NetworkManagerEvent> + Send + 'static>>,
>;

pub(crate) struct StreamMap {
    map: InnerMap,
    tx: UnboundedSender<NetworkManagerEvent>,
}
unsafe impl Sync for StreamMap {}

const MANUAL: &str = "MANUAL";

impl StreamMap {
    pub(crate) fn new() -> Self {
        let mut map = InnerMap::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<NetworkManagerEvent>();
        map.insert(MANUAL, UnboundedReceiverStream::new(rx).boxed());

        Self { map, tx }
    }

    pub(crate) fn add<S>(&mut self, name: &'static str, stream: S)
    where
        S: Stream<Item = NetworkManagerEvent> + Send + 'static,
    {
        let stream = stream.boxed();
        self.map.insert(name, stream);
    }

    pub(crate) fn remove(&mut self, name: &'static str) {
        self.map.remove(name);
    }

    pub(crate) async fn next(&mut self) -> Option<(&'static str, NetworkManagerEvent)> {
        self.map.next().await
    }

    pub(crate) fn emit(&self, event: NetworkManagerEvent) -> Result<()> {
        self.tx
            .send(event)
            .context("failed to self-send message; closed stream")
    }
}
