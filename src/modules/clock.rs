use crate::{
    Event,
    event_queue::EventQueue,
    utils::{StringRef, StringRefExt},
};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick() {
        let now = StringRef::new(
            &chrono::Local::now()
                .format("%H:%M:%S | %b %d | %a")
                .to_string(),
        );

        EventQueue::push_back(Event::Time { now });
    }
}
