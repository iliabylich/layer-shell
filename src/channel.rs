use crate::{Command, Event, fatal, fd_id::FdId};
use libc::{PF_LOCAL, SOCK_STREAM, close, read, socketpair, write};
use mio::Token;
use std::{
    os::fd::{AsRawFd, RawFd},
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

pub(crate) struct VerboseSender<T> {
    tx: Sender<T>,
}

impl<T> Clone for VerboseSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T> VerboseSender<T> {
    pub(crate) fn send(&self, t: T) {
        if let Err(err) = self.tx.send(t) {
            log::error!("failed to send through channel: {:?}", err);
        }
    }
}

pub(crate) struct VerboseReceiver<T> {
    rx: Receiver<T>,
}

impl<T> VerboseReceiver<T> {
    pub(crate) fn recv(&mut self) -> Option<T> {
        match self.rx.try_recv() {
            Ok(t) => Some(t),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::error!("channel is closed, can't recv");
                None
            }
        }
    }
}

pub(crate) struct EventsChannel {
    pub(crate) tx: VerboseSender<Event>,
    pub(crate) rx: VerboseReceiver<Event>,
}

impl EventsChannel {
    pub(crate) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx: VerboseSender { tx },
            rx: VerboseReceiver { rx },
        }
    }
}

pub(crate) struct SignalingSender<T> {
    tx: VerboseSender<T>,
    fd: RawFd,
}

impl<T> Clone for SignalingSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            fd: self.fd,
        }
    }
}

impl<T> SignalingSender<T> {
    pub(crate) fn signal_and_send(&self, t: T) {
        let res = unsafe { write(self.fd, (&1_u8 as *const u8).cast(), 1) };
        assert_ne!(res, -1, "last err: {}", std::io::Error::last_os_error());
        self.tx.send(t);
    }
}

impl<T> Drop for SignalingSender<T> {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub(crate) struct SignalingCommandReceiver {
    rx: VerboseReceiver<Command>,
    fd: RawFd,
}

impl SignalingCommandReceiver {
    pub(crate) const TOKEN: Token = FdId::Command.token();
}

impl AsRawFd for SignalingCommandReceiver {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl SignalingCommandReceiver {
    pub(crate) fn consume_signal(&mut self) {
        let mut signal = 0;
        unsafe { read(self.fd, (&mut signal as *mut i32).cast(), 1) };
    }
    pub(crate) fn recv(&mut self) -> Option<Command> {
        self.rx.recv()
    }
}

impl Drop for SignalingCommandReceiver {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub(crate) struct CommandsChannel {
    pub(crate) tx: SignalingSender<Command>,
    pub(crate) rx: Option<SignalingCommandReceiver>,
}

impl CommandsChannel {
    pub(crate) fn new() -> Self {
        let mut fd = [0_i32; 2];
        let res = unsafe { socketpair(PF_LOCAL, SOCK_STREAM, 0, fd.as_mut_ptr()) };
        if res == -1 {
            fatal!(
                "failed to call socketpair: {:?}",
                std::io::Error::last_os_error()
            );
        }
        let [writerfd, readerfd] = fd;
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx: SignalingSender {
                tx: VerboseSender { tx },
                fd: writerfd,
            },
            rx: Some(SignalingCommandReceiver {
                rx: VerboseReceiver { rx },
                fd: readerfd,
            }),
        }
    }

    pub(crate) fn take_rx(&mut self) -> SignalingCommandReceiver {
        self.rx
            .take()
            .unwrap_or_else(|| fatal!("can't take receiver twice"))
    }
}
