use crate::{
    scheduler::{actor::Action, timer::Timer, Actor},
    Command,
};
use min_max_heap::MinMaxHeap;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub(crate) struct QueueItem {
    pub(crate) name: &'static str,
    pub(crate) tick_timer: Option<Timer>,
    pub(crate) exec_timer: Option<Timer>,
    pub(crate) module: Box<dyn Actor>,
    pub(crate) rx: Receiver<Command>,
}

impl QueueItem {
    fn first_timer_and_action(&self) -> Option<(Timer, Action)> {
        match (self.tick_timer, self.exec_timer) {
            (None, None) => None,
            (None, Some(exec)) => Some((exec, Action::Exec)),
            (Some(tick), None) => Some((tick, Action::Tick)),
            (Some(tick), Some(exec)) => {
                if tick < exec {
                    Some((tick, Action::Tick))
                } else {
                    Some((exec, Action::Exec))
                }
            }
        }
    }

    fn first_timer(&self) -> Option<Timer> {
        let (timer, _) = self.first_timer_and_action()?;
        Some(timer)
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let lhs = self.first_timer().map(|timer| timer.ts).unwrap_or(u64::MAX);
        let rhs = other
            .first_timer()
            .map(|timer| timer.ts)
            .unwrap_or(u64::MAX);
        lhs.cmp(&rhs)
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.first_timer() == other.first_timer()
    }
}

impl Eq for QueueItem {}

#[derive(Debug)]
pub(crate) struct Queue {
    heap: MinMaxHeap<QueueItem>,
}

impl Queue {
    pub(crate) fn new() -> Self {
        Self {
            heap: MinMaxHeap::new(),
        }
    }

    pub(crate) fn register(
        &mut self,
        name: &'static str,
        module: Box<dyn Actor>,
        rx: Receiver<Command>,
    ) {
        let item = QueueItem {
            name,
            tick_timer: Some(Timer::default_tick()),
            exec_timer: Some(Timer::default_exec()),
            module,
            rx,
        };
        self.heap.push(item);
    }

    pub(crate) fn push(&mut self, item: QueueItem) {
        self.heap.push(item);
    }

    pub(crate) fn pop_ready(&mut self) -> Option<(QueueItem, Action)> {
        if let Some(item) = self.heap.pop_min() {
            if let Some((timer, action)) = item.first_timer_and_action() {
                if timer.in_the_past() {
                    return Some((item, action));
                }
            }
            self.heap.push(item);
        }

        None
    }
}
