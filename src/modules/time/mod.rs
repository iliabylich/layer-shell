use crate::Event;

pub(crate) fn tick() {
    let now = chrono::Local::now();
    let event = Event::Time {
        time: now.format("%H:%M:%S").to_string().into(),
        date: now.format("%Y %B %e").to_string().into(),
    };
    event.emit();
}
