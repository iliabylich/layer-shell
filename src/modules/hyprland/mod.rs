use crate::{hyprctl, scheduler::Actor, Command};
use anyhow::{bail, Context, Result};
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, ErrorKind, Lines},
    ops::ControlFlow,
    os::unix::net::UnixStream,
    time::Duration,
};

mod connection;
mod raw_event;
mod state;

pub(crate) struct Hyprland {
    state: State,
    reader: Lines<BufReader<UnixStream>>,
}

impl Actor for Hyprland {
    fn name() -> &'static str {
        "Hyprland"
    }

    fn start() -> Result<Box<dyn Actor>> {
        let reader = connection::reader()?;

        let state = State::new()?;
        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let reader = BufReader::new(reader).lines();

        Ok(Box::new(Hyprland { state, reader }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        loop {
            let data = self.reader.next().context("Hyprland socket is closed")?;
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

        Ok(ControlFlow::Continue(Duration::from_millis(50)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        if let Command::HyprlandGoToWorkspace { idx } = cmd {
            hyprctl::dispatch(format!("workspace {}", *idx + 1))?;
        }

        Ok(ControlFlow::Continue(()))
    }
}

impl std::fmt::Debug for Hyprland {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hyprland")
            .field("state", &self.state)
            .field("reader", &"<reader>")
            .finish()
    }
}
