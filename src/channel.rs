use crate::{
    Command, Event,
    epoll::{FdId, Reader},
    fatal,
};
use libc::{PF_LOCAL, SOCK_STREAM, close, read, socketpair, write};
use std::{
    os::fd::AsRawFd,
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
    pub(crate) tx: Option<VerboseSender<Event>>,
    pub(crate) rx: VerboseReceiver<Event>,
}

impl EventsChannel {
    pub(crate) fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx: Some(VerboseSender { tx }),
            rx: VerboseReceiver { rx },
        }
    }

    pub(crate) fn take_tx(&mut self) -> VerboseSender<Event> {
        self.tx
            .take()
            .unwrap_or_else(|| fatal!("can't take sender twice"))
    }
}

pub(crate) struct SignalingSender<T> {
    tx: VerboseSender<T>,
    fd: i32,
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
    fd: i32,
}

impl Reader for SignalingCommandReceiver {
    type Output = Vec<Command>;

    const NAME: &str = "Commands channel";

    fn read(&mut self) -> anyhow::Result<Self::Output> {
        unreachable!("use direct API instead")
    }

    fn fd(&self) -> i32 {
        self.fd
    }

    fn fd_id(&self) -> FdId {
        FdId::Command
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

impl AsRawFd for SignalingCommandReceiver {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd
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
