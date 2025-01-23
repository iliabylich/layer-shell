#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Item {
    pub(crate) service: String,
    pub(crate) path: String,
    pub(crate) menu_path: String,
}
