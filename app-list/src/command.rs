use crate::State;

#[derive(Debug)]
pub struct AppListReset;

impl AppListReset {
    pub async fn exec(self) {
        State::instance().reset().await;
    }
}

#[derive(Debug)]
pub struct AppListGoUp;

impl AppListGoUp {
    pub async fn exec(self) {
        State::instance().go_up().await;
    }
}

#[derive(Debug)]
pub struct AppListGoDown;

impl AppListGoDown {
    pub async fn exec(self) {
        State::instance().go_down().await;
    }
}

#[derive(Debug)]
pub struct AppListSetSearch {
    pub search: String,
}

impl AppListSetSearch {
    pub async fn exec(self) {
        State::instance().set_search(self.search).await;
    }
}

#[derive(Debug)]
pub struct AppListExecSelected;

impl AppListExecSelected {
    pub async fn exec(self) {
        State::instance().exec_selected().await;
    }
}
