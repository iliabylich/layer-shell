use async_stream::stream;
use futures::Stream;

use crate::Event;

pub(crate) fn connect() -> impl Stream<Item = Event> {
    stream! {
        loop {
            yield now();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

fn now() -> Event {
    let now = chrono::Local::now();
    Event::Time {
        time: now.format("%H:%M:%S").to_string().into(),
        date: now.format("%Y %B %e").to_string().into(),
    }
}
