use crate::App;

#[derive(Debug)]
pub enum Event {
    AppList(AppList),
}

#[derive(Debug)]
pub struct AppList {
    pub apps: Vec<App>,
}
