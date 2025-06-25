use crate::{Event, store::Store};
use futures::{Stream, ready};
use pin_project_lite::pin_project;
use std::time::Duration;
use tokio::time::{Interval, interval};

pin_project! {
    pub struct CPU {
        #[pin]
        timer: Interval,
        store: Store
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            timer: interval(Duration::from_secs(1)),
            store: Store::new(),
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for CPU {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        let _ = ready!(this.timer.poll_tick(cx));

        match this.store.update() {
            Ok(usage_per_core) => std::task::Poll::Ready(Some(Event { usage_per_core })),
            Err(err) => {
                log::error!("CPU stream has crashes: {err:?}");
                std::task::Poll::Ready(None)
            }
        }
    }
}
