use crate::{Event, channel::EventSender, modules::TickingModule};

pub(crate) struct Clock {
    tx: EventSender,
}

impl Clock {
    pub(crate) const INTERVAL: u64 = 1;

    pub(crate) fn new(tx: &EventSender) -> Self {
        Self { tx: tx.clone() }
    }
}

impl TickingModule for Clock {
    const NAME: &str = "Time";

    fn tick(&mut self) -> anyhow::Result<()> {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S | %b %e | %a").to_string(),
        };
        self.tx.send(event);
        Ok(())
    }
}
