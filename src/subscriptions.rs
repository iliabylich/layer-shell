use crate::{fatal::fatal, Event};
use std::sync::{LazyLock, Mutex};

pub(crate) struct Subscriptions;

type F = extern "C" fn(*const Event);

static SUBSCRIPTIONS: LazyLock<Mutex<Vec<F>>> = LazyLock::new(|| Mutex::new(vec![]));

impl Subscriptions {
    pub(crate) fn add(f: F) {
        let mut subscriptions = SUBSCRIPTIONS
            .lock()
            .unwrap_or_else(|_| fatal!("lock is poisoned"));
        subscriptions.push(f);
    }

    pub(crate) fn call_each(event: *const Event) {
        let subscriptions = {
            SUBSCRIPTIONS
                .lock()
                .unwrap_or_else(|_| fatal!("lock is poisoned"))
                .clone()
        };

        for sub in subscriptions.iter() {
            sub(event);
        }
    }
}
