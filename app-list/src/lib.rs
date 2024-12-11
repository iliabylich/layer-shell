use async_stream::stream;
use futures::Stream;

mod app;
mod app_icon;
mod command;
mod event;
mod state;
mod system_app;

pub use app::App;
pub use app_icon::AppIcon;
pub use command::Command;
pub use event::{AppList, Event};

pub(crate) use state::State;

pub async fn connect() -> impl Stream<Item = Event> {
    stream! {
        let mut rx = State::setup();

        loop {
            while let Some(event) = rx.recv().await {
                yield event;
            }

            println!("Post loop");

            // tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
}

pub async fn on_command(command: &Command) {
    let state = State::instance();

    match command {
        Command::Reset => state.reset().await,
        Command::GoUp => state.go_up().await,
        Command::GoDown => state.go_down().await,
        Command::SetSearch(search) => state.set_search(search).await,
        Command::ExecSelected => state.exec_selected().await,
    }
}
