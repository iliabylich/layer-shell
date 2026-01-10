use crate::{Event, timerfd::Tick};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn on_tick(tick: Tick, events: &mut Vec<Event>) {
        if tick.is_multiple_of(1) {
            let time = chrono::Local::now()
                .format("%H:%M:%S | %b %e | %a")
                .to_string();
            events.push(Event::Clock { time: time.into() });
        }
    }
}
