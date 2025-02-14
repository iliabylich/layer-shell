use crate::{hyprctl, Event, VerboseSender};
use anyhow::Result;
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, Lines},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

mod connection;
mod raw_event;
mod state;

pub(crate) struct ConnectedHyprland {
    fd: i32,
    reader: Lines<BufReader<UnixStream>>,
    state: State,
    tx: VerboseSender<Event>,
}

impl ConnectedHyprland {
    fn try_new(tx: VerboseSender<Event>) -> Result<Self> {
        let socket = connection::connect_to_socket()?;
        let fd = socket.as_raw_fd();
        let reader = BufReader::new(socket).lines();
        let state = State::new()?;

        tx.send(state.as_language_changed_event());
        tx.send(state.as_workspaces_changed_event());

        Ok(Self {
            fd,
            reader,
            state,
            tx,
        })
    }

    fn read(&mut self) {
        while let Some(Ok(line)) = self.reader.next() {
            if let Some(event) = RawEvent::parse(&line) {
                let event = self.state.apply(event);
                self.tx.send(event);
            }
        }
    }
}

pub(crate) enum Hyprland {
    Connected(ConnectedHyprland),
    Disconnected,
}

impl Hyprland {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        ConnectedHyprland::try_new(tx)
            .map(Self::Connected)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(inner) = self {
            inner.read();
        }
    }

    pub(crate) fn go_to_workspace(&self, idx: usize) {
        if let Err(err) = hyprctl::dispatch(format!("workspace {}", idx + 1)) {
            log::error!("{:?}", err)
        }
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        match self {
            Self::Connected(ConnectedHyprland { fd, .. }) => Some(*fd),
            Self::Disconnected => None,
        }
    }
}
