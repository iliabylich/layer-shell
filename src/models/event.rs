use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    Cpu {
        usage_per_core: Vec<usize>,
    },
    Time {
        time: String,
        date: String,
    },
    Workspaces {
        ids: HashSet<usize>,
        active_id: usize,
    },
    Language {
        lang: String,
    },
}
