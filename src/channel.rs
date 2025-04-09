use crate::{Command, Event, fatal, fd_id::FdId};
use mio::Token;
use std::{
    io::{Read, Write},
    os::fd::{AsRawFd, RawFd},
    sync::mpsc::TryRecvError,
};

#[derive(Clone)]
pub(crate) struct EventSender(std::sync::mpsc::Sender<Event>);
impl EventSender {
    pub(crate) fn send(&self, e: Event) {
        if let Err(err) = self.0.send(e) {
            log::error!("failed to send event through channel: {:?}", err);
        }
    }
}
pub(crate) struct EventReceiver(std::sync::mpsc::Receiver<Event>);
impl EventReceiver {
    pub(crate) fn recv(&self) -> Option<Event> {
        match self.0.try_recv() {
            Ok(t) => Some(t),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::error!("channel is closed, can't recv");
                None
            }
        }
    }
}

pub(crate) struct CommandSender(mio::unix::pipe::Sender, std::sync::mpsc::Sender<Command>);
impl CommandSender {
    pub(crate) fn send(&mut self, c: Command) {
        if let Err(err) = self.1.send(c) {
            log::error!("failed to send event through channel: {:?}", err);
            return;
        }
        if let Err(err) = self.0.write(&[1]) {
            log::error!("failed to write notification about command: {:?}", err);
        }
    }
}

pub(crate) struct CommandReceiver(
    mio::unix::pipe::Receiver,
    std::sync::mpsc::Receiver<Command>,
);
impl CommandReceiver {
    pub(crate) const TOKEN: Token = FdId::Command.token();

    pub(crate) fn consume_signal(&mut self) {
        let mut buf = [0; 1];
        if let Err(err) = self.0.read_exact(&mut buf) {
            log::error!("failed to read notification about command: {:?}", err);
        }
    }
    pub(crate) fn recv(&self) -> Option<Command> {
        match self.1.try_recv() {
            Ok(t) => Some(t),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::error!("channel is closed, can't recv");
                None
            }
        }
    }
}

impl AsRawFd for CommandReceiver {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

pub(crate) fn events() -> (EventSender, EventReceiver) {
    let (tx, rx) = std::sync::mpsc::channel();
    (EventSender(tx), EventReceiver(rx))
}

pub(crate) fn commands() -> (CommandSender, CommandReceiver) {
    let (tx0, rx0) =
        mio::unix::pipe::new().unwrap_or_else(|err| fatal!("failed to create pipe: {err:?}"));
    let (tx1, rx1) = std::sync::mpsc::channel();

    (CommandSender(tx0, tx1), CommandReceiver(rx0, rx1))
}
