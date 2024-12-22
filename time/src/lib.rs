use async_stream::stream;
use futures::Stream;

#[derive(Debug)]
pub struct Time {
    pub date: String,
    pub time: String,
}

pub fn connect() -> impl Stream<Item = Time> {
    stream! {
        loop {
            yield now();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

fn now() -> Time {
    let now = chrono::Local::now();
    Time {
        time: now.format("%H:%M:%S").to_string(),
        date: now.format("%Y %B %e").to_string(),
    }
}
