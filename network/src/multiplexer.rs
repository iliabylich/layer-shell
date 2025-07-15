use crate::nm_event::NetworkManagerEvent;
use futures::{Stream, ready, stream::BoxStream};
use std::pin::Pin;
use tokio_stream::StreamMap;

pub(crate) struct Multiplexer {
    map: StreamMap<&'static StreamId, BoxStream<'static, NetworkManagerEvent>>,
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) struct StreamId {
    pub(crate) name: &'static str,
    pub(crate) children: &'static [&'static StreamId],
}

impl Multiplexer {
    pub(crate) fn new() -> Self {
        Self {
            map: StreamMap::new(),
        }
    }

    pub(crate) fn add(
        &mut self,
        id: &'static StreamId,
        stream: BoxStream<'static, NetworkManagerEvent>,
    ) {
        self.map.insert(id, stream);
    }

    pub(crate) fn remove_with_children(&mut self, id: &'static StreamId) {
        self.map.remove(id);

        for child in id.children {
            self.remove_with_children(child);
        }
    }
}

impl Stream for Multiplexer {
    type Item = (&'static str, NetworkManagerEvent);

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let map = Pin::new(&mut self.as_mut().get_mut().map);
        let out = ready!(map.poll_next(cx)).map(|(stream_id, event)| (stream_id.name, event));
        std::task::Poll::Ready(out)
    }
}
