use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Timer {
    pub(crate) ts: u64,
    pub(crate) interval: Duration,
}

impl Timer {
    pub(crate) fn start_now(interval: Duration) -> Self {
        Self {
            ts: now(),
            interval,
        }
    }

    pub(crate) fn tick(&mut self) {
        self.ts = now() + self.interval.as_millis() as u64;
    }

    pub(crate) fn in_the_past(&self) -> bool {
        self.ts < now()
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ts.cmp(&other.ts)
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.ts == other.ts
    }
}

impl Eq for Timer {}

fn now() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
