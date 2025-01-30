use crate::{
    hyprctl,
    scheduler::{Module, RepeatingModule},
    Command,
};
use anyhow::{bail, Context, Result};
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, ErrorKind, Lines},
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

impl Module for Hyprland {
    const NAME: &str = "Hyprland";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let reader = connection::reader()?;

        let state = State::new()?;
        state.as_language_changed_event().emit();
        state.as_workspaces_changed_event().emit();

        let reader = BufReader::new(reader).lines();

        Ok(Some(Box::new(Hyprland { state, reader })))
    }
}

impl RepeatingModule for Hyprland {
    fn tick(&mut self) -> Result<Duration> {
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

        Ok(Duration::from_millis(50))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
        match cmd {
            Command::HyprlandGoToWorkspace { idx } => {
                hyprctl::dispatch(format!("workspace {}", *idx + 1))?;
            }

            _ => {}
        }

        Ok(())
    }
}
