#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Item {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) menu_path: String,
}
