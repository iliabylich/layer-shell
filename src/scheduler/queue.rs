use crate::{macros::fatal, scheduler::queue_item::QueueItem};
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

    pub(crate) fn push(name: &'static str, run_at: u64) {
        let Ok(mut global) = QUEUE.lock() else {
            fatal!("lock is poisoned");
        };
        let Some(queue) = global.as_mut() else {
            fatal!("Queue::init() hasn't been called");
        };
        queue.push(QueueItem { name, run_at });
    }

    pub(crate) fn pop_min_lt(value: u64) -> Option<&'static str> {
        let Ok(mut global) = QUEUE.lock() else {
            fatal!("lock is poisoned");
        };
        let Some(queue) = global.as_mut() else {
            fatal!("Queue::init() hasn't been called");
        };

        match queue.peek_min().copied() {
            Some(QueueItem { run_at, name }) if run_at < value => {
                queue.pop_min();
                Some(name)
            }
            _ => None,
        }
    }
}
