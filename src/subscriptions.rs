use crate::{macros::fatal, Event};
use std::{
    ffi::c_void,
    sync::{LazyLock, Mutex},
};

pub(crate) struct Subscriptions;

type F = extern "C" fn(*const Event, *mut c_void);

#[derive(Clone)]
struct Data {
    ptr: *mut c_void,
}
unsafe impl Send for Data {}

static SUBSCRIPTIONS: LazyLock<Mutex<Vec<(F, Data)>>> = LazyLock::new(|| Mutex::new(vec![]));

impl Subscriptions {
    pub(crate) fn add(f: F, data: *mut c_void) {
        let mut subscriptions = SUBSCRIPTIONS
            .lock()
            .unwrap_or_else(|_| fatal!("lock is poisoned"));
        subscriptions.push((f, Data { ptr: data }));
    }

    pub(crate) fn call_each(event: *const Event) {
        let subscriptions = {
            SUBSCRIPTIONS
                .lock()
                .unwrap_or_else(|_| fatal!("lock is poisoned"))
                .clone()
        };

        for (sub, data) in subscriptions.into_iter() {
            sub(event, data.ptr);
        }
    }
}
