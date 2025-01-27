use crate::{
    macros::fatal,
    scheduler::{queue_item::QueueItem, RepeatingModule},
};
use min_max_heap::MinMaxHeap;
use std::sync::Mutex;

static QUEUE: Mutex<Option<MinMaxHeap<QueueItem>>> = Mutex::new(None);

pub(crate) struct Queue;

impl Queue {
    pub(crate) fn init() {
        let Ok(mut global) = QUEUE.lock() else {
            fatal!("lock is poisoned");
        };

        *global = Some(MinMaxHeap::new());
    }

    pub(crate) fn push(name: &'static str, run_at: u64, module: Box<dyn RepeatingModule>) {
        let Ok(mut global) = QUEUE.lock() else {
            fatal!("lock is poisoned");
        };
        let Some(queue) = global.as_mut() else {
            fatal!("Queue::init() hasn't been called");
        };
        queue.push(QueueItem {
            name,
            run_at,
            module,
        });
    }

    pub(crate) fn pop_min_lt(value: u64) -> Option<(&'static str, Box<dyn RepeatingModule>)> {
        let Ok(mut global) = QUEUE.lock() else {
            fatal!("lock is poisoned");
        };
        let Some(queue) = global.as_mut() else {
            fatal!("Queue::init() hasn't been called");
        };

        if queue.peek_min().is_some_and(|item| item.run_at < value) {
            let item = queue.pop_min().expect("bug");
            Some((item.name, item.module))
        } else {
            None
        }
    }
}
