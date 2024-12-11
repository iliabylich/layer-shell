use std::collections::HashSet;

#[derive(Debug)]
pub enum Event {
    Workspaces(Workspaces),
    Language(Language),
}

#[derive(Debug)]
pub struct Workspaces {
    pub ids: HashSet<usize>,
    pub active_id: usize,
}

#[derive(Debug)]
pub struct Language {
    pub lang: String,
}
