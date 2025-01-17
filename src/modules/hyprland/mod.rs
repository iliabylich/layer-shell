use crate::fatal::fatal;
use std::io::{BufRead as _, BufReader};

mod command;
mod connection;
mod raw_event;
mod state;

pub(crate) use command::go_to_workspace;

pub(crate) fn setup() {
    std::thread::spawn(move || {
        let socket = connection::connect_to_socket()
            .unwrap_or_else(|err| fatal!("failed to connect to Hyprland socket: {:?}", err));

        let mut state = state::State::new()
            .unwrap_or_else(|err| fatal!("failed to get initial Hyprland state: {:?}", err));

        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let buffered = BufReader::new(socket);
        let mut lines = buffered.lines();

        while let Some(Ok(line)) = lines.next() {
            if let Some(event) = raw_event::RawEvent::parse(&line) {
                let event = state.apply(event);
                event.emit();
            }
        }
    });
}
