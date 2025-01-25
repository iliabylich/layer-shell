use crate::scheduler::Module;
use anyhow::{bail, Context, Result};
use raw_event::RawEvent;
use state::State;
use std::{
    any::Any,
    io::{BufRead as _, BufReader, ErrorKind, Lines},
    os::unix::net::UnixStream,
};

mod command;
mod connection;
mod raw_event;
mod state;

type Reader = Lines<BufReader<UnixStream>>;

pub(crate) struct Hyprland;

impl Hyprland {
    pub(crate) fn go_to_workspace(idx: usize) -> Result<()> {
        command::go_to_workspace(idx)
    }
}

impl Module for Hyprland {
    const NAME: &str = "Hyprland";
    const INTERVAL: Option<u64> = Some(50);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        let socket = connection::connect_to_socket()?;

        let state = State::new()?;
        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let buffered = BufReader::new(socket);
        let lines = buffered.lines();

        let state: (State, Reader) = (state, lines);

        Ok(Box::new(state))
    }

    fn tick(state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let (state, reader) = state
            .downcast_mut::<(State, Reader)>()
            .context("Hyprland state is malformed")?;

        loop {
            let data = reader.next().context("Hyprland socket is closed")?;
            match data {
                Ok(line) => {
                    if let Some(event) = RawEvent::parse(&line) {
                        let event = state.apply(event);
                        event.emit();
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // all good, there's no data left for now
                    return Ok(());
                }
                Err(err) => {
                    bail!("Hyprland IO error: {:?}", err);
                }
            }
        }
    }
}
