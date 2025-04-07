use crate::Event;

pub type Data = *mut std::ffi::c_void;
pub type SubscriptionFn = extern "C" fn(&Event, Data);

pub struct Subscriptions {
    list: Vec<(SubscriptionFn, Data)>,
}

#[unsafe(no_mangle)]
pub extern "C" fn io_subscription_list_new() -> *mut Subscriptions {
    Box::leak(Box::new(Subscriptions { list: vec![] }))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_subscription_list_add(
    subscriptions: &mut Subscriptions,
    f: SubscriptionFn,
    data: Data,
) {
    subscriptions.list.push((f, data));
}

impl Subscriptions {
    pub(crate) fn notify_each(&self, event: &Event) {
        for (f, data) in self.list.iter() {
            (f)(event, *data);
        }
    }
}
