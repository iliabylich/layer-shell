use futures_util::{Stream, ready};
use pin_project_lite::pin_project;
use std::{task::Poll::*, time::Duration};
use tokio::time::{Interval, interval};

use crate::event::Event;

pin_project! {
    pub struct ClockStream {
        #[pin]
        timer: Interval
    }
}

impl ClockStream {
    pub fn new() -> Self {
        Self {
            timer: interval(Duration::from_secs(1)),
        }
    }
}

impl Default for ClockStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for ClockStream {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        let _ = ready!(this.timer.poll_tick(cx));

        let now = chrono::Local::now();
        Ready(Some(Event {
            time: now.format("%H:%M:%S | %b %e | %a").to_string(),
        }))
    }
}
