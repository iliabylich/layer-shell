use crate::{models::Event, utils::global};

global!(SUBSCRIPTIONS, Vec<fn(&Event)>);

pub(crate) fn subscribe(f: fn(&Event)) {
    SUBSCRIPTIONS::get().push(f);
}

pub(crate) fn all() -> &'static mut Vec<fn(&Event)> {
    SUBSCRIPTIONS::get()
}

pub(crate) fn init() {
    SUBSCRIPTIONS::set(vec![])
}
