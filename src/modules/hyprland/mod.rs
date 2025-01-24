use crate::scheduler::Module;
use anyhow::Result;
use std::io::{BufRead as _, BufReader};

mod command;
mod connection;
mod raw_event;
mod state;

pub(crate) use command::go_to_workspace;

pub(crate) struct Hyprland;

impl Module for Hyprland {
    const NAME: &str = "Hyprland";

    fn start() -> Result<Option<(u64, fn() -> Result<()>)>> {
        let socket = connection::connect_to_socket()?;
        let mut state = state::State::new()?;

        std::thread::spawn(move || {
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

        Ok(None)
    }
}
