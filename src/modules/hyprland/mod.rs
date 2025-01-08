use std::io::{BufRead as _, BufReader};

mod command;
mod connection;
mod raw_event;
mod state;

pub(crate) use command::go_to_workspace;

pub(crate) fn setup() {
    std::thread::spawn(move || {
        let socket = match connection::connect_to_socket() {
            Ok(socket) => socket,
            Err(err) => {
                log::error!("failed to connect to Hyprland socket: {:?}", err);
                std::process::exit(1)
            }
        };

        let mut state = match state::State::new() {
            Ok(state) => state,
            Err(err) => {
                log::error!("failed to get initial Hyprland state: {:?}", err);
                std::process::exit(1)
            }
        };

        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let buffered = BufReader::new(socket);
        let mut lines = buffered.lines();

        while let Some(Ok(line)) = lines.next() {
            if let Some(event) = raw_event::RawEvent::parse(line) {
                let event = state.apply(event);
                event.emit();
            }
        }
    });
}
