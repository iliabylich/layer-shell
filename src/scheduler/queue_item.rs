use crate::scheduler::RepeatingModule;

pub(crate) struct QueueItem {
    pub(crate) name: &'static str,
    pub(crate) run_at: u64,
    pub(crate) module: Box<dyn RepeatingModule + 'static>,
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.run_at.cmp(&other.run_at)
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.run_at == other.run_at
    }
}

impl Eq for QueueItem {}

impl std::fmt::Debug for QueueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItem")
            .field("name", &self.name)
            .field("run_at", &self.run_at)
            .field("module", &"<module>")
            .finish()
    }
}
