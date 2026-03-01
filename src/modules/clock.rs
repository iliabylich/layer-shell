use crate::Event;

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick(events: &mut Vec<Event>) {
        events.push(Event::Clock {
            unix_seconds: chrono::Local::now().timestamp(),
        });
    }
}
