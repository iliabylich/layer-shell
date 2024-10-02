use crate::Event;
use layer_shell_utils::global;

global!(SUBSCRIPTIONS, Vec<fn(&Event)>);

pub fn subscribe(f: fn(&Event)) {
    SUBSCRIPTIONS::get().push(f);
}

pub(crate) fn all() -> &'static mut Vec<fn(&Event)> {
    SUBSCRIPTIONS::get()
}

pub(crate) fn init() {
    SUBSCRIPTIONS::set(vec![])
}
