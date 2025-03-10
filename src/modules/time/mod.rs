use crate::{Event, channel::VerboseSender};

pub(crate) struct Time {
    tx: VerboseSender<Event>,
}

impl Time {
    pub(crate) const INTERVAL: u64 = 1;

    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        Self { tx }
    }

    pub(crate) fn tick(&mut self) {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S | %b %e | %a").to_string().into(),
        };
        self.tx.send(event)
    }
}
