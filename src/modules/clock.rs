use crate::{Event, event_queue::EventQueue};

pub(crate) struct Clock {
    events: EventQueue,
}

impl Clock {
    pub(crate) fn new(events: EventQueue) -> Self {
        Self { events }
    }

    pub(crate) fn tick(&self) {
        self.events.push_back(Event::Clock {
            unix_seconds: chrono::Local::now().timestamp(),
        });
    }
}
