#[derive(Debug)]
pub enum Event {
    Workspaces { ids: Vec<usize>, active_id: usize },
    Language { lang: String },
}
