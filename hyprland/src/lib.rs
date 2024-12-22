use async_stream::stream;
use futures::{pin_mut, Stream, StreamExt};

mod command;
mod event;
mod raw_event;
mod raw_stream;
mod state;

pub use command::HyprlandGoToWorkspace;
pub use event::{HyprlandEvent, Language, Workspaces};
use state::State;

pub async fn connect() -> impl Stream<Item = HyprlandEvent> {
    stream! {
        match State::new().await {
            Ok(mut state) => {
                let raw_stream = crate::raw_stream::raw_events_stream().await;
                pin_mut!(raw_stream);

                yield state.as_workspaces_changed_event();
                yield state.as_language_changed_event();

                while let Some(raw_event) = raw_stream.next().await {
                    let event = state.apply(raw_event);
                    yield event;
                }
            }
            Err(err) => {
                log::error!("failed to get initial Hyprland state: {:?}", err);
            }
        }

    }
}
