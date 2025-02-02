use crate::{
    scheduler::{
        actor::{Action, ExecutionPlan},
        Actor,
    },
    Command,
};
use min_max_heap::MinMaxHeap;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub(crate) struct QueueItem {
    pub(crate) name: &'static str,
    pub(crate) execution_plan: ExecutionPlan,
    pub(crate) module: Box<dyn Actor>,
    pub(crate) rx: Receiver<Command>,
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.execution_plan.cmp(&other.execution_plan)
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.execution_plan == other.execution_plan
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
            execution_plan: ExecutionPlan::initial(),
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
            let (timer, action) = item.execution_plan.first_timer_and_action();
            if timer.in_the_past() {
                Some((item, action))
            } else {
                self.heap.push(item);
                None
            }
        } else {
            None
        }
    }
}
