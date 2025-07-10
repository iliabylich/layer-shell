use crate::nm_event::NetworkManagerEvent;
use futures::{Stream, StreamExt, ready, stream::BoxStream};
use pin_project_lite::pin_project;
use std::pin::Pin;
use tokio_stream::StreamMap as TokioStreamMap;

pin_project! {
    pub(crate) struct StreamMap {
        #[pin]
        map: TokioStreamMap<&'static StreamId, BoxStream<'static, NetworkManagerEvent>>,
    }
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) struct StreamId {
    name: &'static str,
    children: &'static [&'static StreamId],
}

macro_rules! stream_id {
    ($name:ident, $children:expr) => {
        pub(crate) static $name: &StreamId = &StreamId {
            name: stringify!($name),
            children: &$children,
        };
    };
}

stream_id!(PRIMARY_CONNECTION, [PRIMARY_DEVICES]);
stream_id!(PRIMARY_DEVICES, [ACCESS_POINT, DEVICE_TX, DEVICE_RX]);
stream_id!(ACCESS_POINT, [ACCESS_POINT_SSID, ACCESS_POINT_STRENGTH]);
stream_id!(ACCESS_POINT_SSID, []);
stream_id!(ACCESS_POINT_STRENGTH, []);
stream_id!(DEVICE_TX, []);
stream_id!(DEVICE_RX, []);
stream_id!(GLOBAL_DEVICES, []);

impl StreamMap {
    pub(crate) fn new() -> Self {
        Self {
            map: TokioStreamMap::new(),
        }
    }

    pub(crate) fn add<S>(&mut self, id: &'static StreamId, stream: S)
    where
        S: Stream<Item = NetworkManagerEvent> + Send + 'static,
    {
        self.map.insert(id, stream.boxed());
    }

    pub(crate) fn remove_with_children(&mut self, id: &'static StreamId) {
        self.map.remove(id);

        for child in id.children {
            self.remove_with_children(child);
        }
    }
}

impl Stream for StreamMap {
    type Item = (&'static str, NetworkManagerEvent);

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let out = ready!(self.project().map.poll_next(cx))
            .map(|(stream_id, event)| (stream_id.name, event));
        std::task::Poll::Ready(out)
    }
}
