use crate::{
    epoll::{FdId, Reader},
    hyprctl,
    modules::maybe_connected::MaybeConnected,
    Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, Lines},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

mod connection;
mod raw_event;
mod state;

pub(crate) struct Hyprland {
    fd: i32,
    reader: Lines<BufReader<UnixStream>>,
    state: State,
    tx: VerboseSender<Event>,
}

impl Hyprland {
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

    pub(crate) fn new(tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(tx))
    }

    pub(crate) fn go_to_workspace(&self, idx: usize) -> Result<()> {
        hyprctl::dispatch(format!("workspace {}", idx + 1)).context("failed to go to workspace")
    }
}

impl Reader for Hyprland {
    type Output = ();

    const NAME: &str = "Hyprland";

    fn read(&mut self) -> Result<Self::Output> {
        while let Some(Ok(line)) = self.reader.next() {
            if let Some(event) = RawEvent::parse(&line) {
                let event = self.state.apply(event);
                self.tx.send(event);
            }
        }
        Ok(())
    }

    fn fd(&self) -> i32 {
        self.fd
    }

    fn fd_id(&self) -> FdId {
        FdId::HyprlandSocket
    }
}
