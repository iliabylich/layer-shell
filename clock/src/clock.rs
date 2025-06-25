use crate::ClockEvent;
use futures::{Stream, ready};
use pin_project_lite::pin_project;
use std::time::Duration;

pin_project! {
    pub struct Clock {
        #[pin]
        timer: tokio::time::Interval,
    }
}

const NAME: &str = "Clock";

impl Clock {
    pub fn new() -> (&'static str, Self) {
        (
            NAME,
            Self {
                timer: tokio::time::interval(Duration::from_secs(1)),
            },
        )
    }
}

impl Stream for Clock {
    type Item = ClockEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        let _ = ready!(this.timer.poll_tick(cx));

        let time = chrono::Local::now()
            .format("%H:%M:%S | %b %e | %a")
            .to_string();

        std::task::Poll::Ready(Some(ClockEvent { time: time.into() }))
    }
}
