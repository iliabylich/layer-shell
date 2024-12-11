use anyhow::Result;
use futures::Stream;

mod command;
mod event;
mod raw_event;
mod raw_stream;
mod state;
mod stateful_stream;

pub use command::HyprlandGoToWorkspace;
pub use event::{Event, Language, Workspaces};

pub async fn connect() -> Result<impl Stream<Item = Event>> {
    stateful_stream::StatefulStream::new().await
}
