use crate::scheduler::{Module, RepeatingModule};
use anyhow::{bail, Context, Result};
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, ErrorKind, Lines},
    os::unix::net::UnixStream,
    time::Duration,
};

mod command;
mod connection;
mod raw_event;
mod state;

pub(crate) struct Hyprland {
    state: State,
    lines: Lines<BufReader<UnixStream>>,
}

impl Hyprland {
    pub(crate) fn go_to_workspace(idx: usize) -> Result<()> {
        command::go_to_workspace(idx)
    }
}

impl Module for Hyprland {
    const NAME: &str = "Hyprland";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let socket = connection::connect_to_socket()?;

        let state = State::new()?;
        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let buffered = BufReader::new(socket);
        let lines = buffered.lines();

        Ok(Some(Box::new(Hyprland { state, lines })))
    }
}

impl RepeatingModule for Hyprland {
    fn tick(&mut self) -> Result<Duration> {
        loop {
            let data = self.lines.next().context("Hyprland socket is closed")?;
            match data {
                Ok(line) => {
                    if let Some(event) = RawEvent::parse(&line) {
                        let event = self.state.apply(event);
                        event.emit();
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // all good, there's no data left for now
                    break;
                }
                Err(err) => {
                    bail!("Hyprland IO error: {:?}", err);
                }
            }
        }

        Ok(Duration::from_millis(50))
    }
}
