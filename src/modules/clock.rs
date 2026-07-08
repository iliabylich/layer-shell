use crate::{
    Event,
    event_queue::EventQueue,
    utils::{StringRef, StringRefExt},
};
use alloc::string::ToString as _;

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick(events: &mut EventQueue) {
        let now = StringRef::new(
            &chrono::Local::now()
                .format("%H:%M:%S | %b %d | %a")
                .to_string(),
        );

        events.push_back(Event::Time { now });
    }
}
