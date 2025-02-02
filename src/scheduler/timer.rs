use std::time::Duration;

use crate::macros::fatal;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Timer {
    pub(crate) ts: u64,
    pub(crate) interval: Duration,
}

impl Timer {
    pub(crate) fn default_tick() -> Self {
        Self {
            ts: now(),
            interval: Duration::from_secs(1),
        }
    }

    pub(crate) fn default_exec() -> Self {
        Self {
            ts: now(),
            interval: Duration::from_millis(50),
        }
    }

    pub(crate) fn tick(&mut self) {
        self.ts = now() + self.interval.as_millis() as u64;
    }

    pub(crate) fn in_the_past(&self) -> bool {
        self.ts < now()
    }

    pub(crate) fn pretty(&self) -> String {
        format!("{}/{:?}ms", self.ts, self.interval)
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
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|err| fatal!("failed to compute current UNIX timestamp: {:?}", err))
        .as_millis() as u64
}
