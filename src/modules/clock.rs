use crate::{Event, event_queue::EventQueue};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick() {
        EventQueue::push_back(Event::Clock {
            unix_seconds: chrono::Local::now().timestamp(),
        });
    }
}
