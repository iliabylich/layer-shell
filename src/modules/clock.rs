use crate::{Event, event_queue::EventQueue};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn tick(&self) {
        EventQueue::push_back(Event::Clock {
            unix_seconds: chrono::Local::now().timestamp(),
        });
    }
}
