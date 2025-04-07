use crate::Event;

pub type Data = *mut std::ffi::c_void;
pub type SubscriptionFn = extern "C" fn(&Event, Data);

pub struct Subscriptions {
    list: Vec<(SubscriptionFn, Data)>,
}

impl Subscriptions {
    pub(crate) fn new() -> Self {
        Self { list: vec![] }
    }

    pub(crate) fn push(
        &mut self,
        f: extern "C" fn(&Event, *mut std::ffi::c_void),
        data: *mut std::ffi::c_void,
    ) {
        self.list.push((f, data));
    }

    pub(crate) fn notify_each(&self, event: &Event) {
        for (f, data) in self.list.iter() {
            (f)(event, *data);
        }
    }
}
