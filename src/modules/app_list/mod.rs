use async_stream::stream;
use futures::Stream;

pub(crate) mod command;
mod state;
mod system_app;

use crate::Event;
use state::State;

pub fn connect() -> impl Stream<Item = Event> {
    stream! {
        let mut rx = State::setup();

        loop {
            while let Some(event) = rx.recv().await {
                yield event;
            }
        }
    }
}
