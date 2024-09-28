use crate::{models::Event, utils::singleton};

struct Subscriptions(Vec<fn(&Event)>);
singleton!(Subscriptions);

pub(crate) fn subscribe(f: fn(&Event)) {
    Subscriptions::get().0.push(f);
}

pub(crate) fn all() -> &'static mut Vec<fn(&Event)> {
    &mut Subscriptions::get().0
}

pub(crate) fn init() {
    Subscriptions::set(Subscriptions(vec![]))
}
