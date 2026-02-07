use crate::Event;

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick(events: &mut Vec<Event>) {
        let time = chrono::Local::now()
            .format("%H:%M:%S | %b %e | %a")
            .to_string();
        events.push(Event::Clock { time: time.into() });
    }
}
