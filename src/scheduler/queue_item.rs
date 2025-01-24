#[derive(Eq, Clone, Copy, Debug)]
pub(crate) struct QueueItem {
    pub(crate) name: &'static str,
    pub(crate) run_at: u64,
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
