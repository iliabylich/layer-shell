use async_stream::stream;
use futures::Stream;

mod app;
mod app_icon;
mod command;
mod state;
mod system_app;

pub use app::App;
pub use app_icon::AppIcon;
pub use command::{
    AppListExecSelected, AppListGoDown, AppListGoUp, AppListReset, AppListSetSearch,
};
pub(crate) use state::State;

#[derive(Debug)]
pub struct AppList {
    pub apps: Vec<App>,
}

pub async fn connect() -> impl Stream<Item = AppList> {
    stream! {
        let mut rx = State::setup();

        loop {
            while let Some(event) = rx.recv().await {
                yield event;
            }
        }
    }
}
