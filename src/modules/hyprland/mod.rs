use crate::{channel::EventSender, fd_id::FdId, hyprctl, modules::Module};
use anyhow::{Context as _, Result};
use mio::net::UnixStream;
use raw_event::RawEvent;
use state::State;
use std::{
    io::{BufRead as _, BufReader, Lines},
    os::fd::{AsRawFd, RawFd},
};

mod connection;
mod raw_event;
mod state;

pub(crate) struct Hyprland {
    fd: RawFd,
    reader: Lines<BufReader<UnixStream>>,
    state: State,
    tx: EventSender,
}

impl Module for Hyprland {
    const FD_ID: FdId = FdId::HyprlandSocket;
    const NAME: &str = "Hyprland";

    type ReadOutput = ();

    fn new(tx: &EventSender) -> Result<Self> {
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
            tx: tx.clone(),
        })
    }

    fn read_events(&mut self) -> Result<()> {
        while let Some(Ok(line)) = self.reader.next() {
            if let Some(event) = RawEvent::parse(&line) {
                let event = self.state.apply(event);
                self.tx.send(event);
            }
        }
        Ok(())
    }
}

impl Hyprland {
    pub(crate) fn go_to_workspace(idx: usize) -> Result<()> {
        hyprctl::dispatch(format!("workspace {}", idx + 1)).context("failed to go to workspace")
    }
}

impl AsRawFd for Hyprland {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
