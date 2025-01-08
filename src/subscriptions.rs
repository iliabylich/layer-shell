use crate::{global::global, Event};

pub(crate) struct Subscriptions;

global!(SUBSCRIPTIONS, Vec<extern "C" fn(*const Event)>);

impl Subscriptions {
    pub(crate) fn setup() {
        SUBSCRIPTIONS::set(vec![]);
    }

    pub(crate) fn add(f: extern "C" fn(*const Event)) {
        SUBSCRIPTIONS::get().push(f);
    }

    pub(crate) fn iter() -> impl Iterator<Item = extern "C" fn(*const Event)> {
        SUBSCRIPTIONS::get().iter().copied()
    }
}
