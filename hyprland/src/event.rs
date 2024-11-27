use std::collections::HashSet;

#[derive(Debug)]
pub enum Event {
    WorkspacesChanged {
        ids: HashSet<usize>,
        active_id: usize,
    },

    LanguageChanged(String),
}
