use crate::{CpuUsageEvent, store::Store};
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

const NAME: &str = "CPU";

impl CPU {
    pub fn new() -> (&'static str, Self) {
        (
            NAME,
            Self {
                timer: interval(Duration::from_secs(1)),
                store: Store::new(),
            },
        )
    }
}

impl Stream for CPU {
    type Item = CpuUsageEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        let _ = ready!(this.timer.poll_tick(cx));

        match this.store.update() {
            Ok(usage_per_core) => std::task::Poll::Ready(Some(CpuUsageEvent {
                usage_per_core: usage_per_core.into(),
            })),
            Err(err) => {
                log::error!(target: "CPU", "{err:?}");
                std::task::Poll::Ready(None)
            }
        }
    }
}
