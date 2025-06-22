use crate::{event::Event, store::Store};
use futures_util::{Stream, ready};
use pin_project_lite::pin_project;
use std::{task::Poll::*, time::Duration};
use tokio::time::Interval;

pin_project! {
    pub struct CpuStream {
        #[pin]
        timer: Interval,

        store: Store
    }
}

impl CpuStream {
    pub fn new() -> Self {
        Self {
            timer: tokio::time::interval(Duration::from_secs(1)),
            store: Store::new(),
        }
    }
}

impl Stream for CpuStream {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        let _ = ready!(this.timer.poll_tick(cx));

        let usage_per_core = match this.store.update() {
            Ok(value) => value,
            Err(err) => {
                log::error!("{err:?}");
                return Ready(None);
            }
        };

        Ready(Some(Event { usage_per_core }))
    }
}
