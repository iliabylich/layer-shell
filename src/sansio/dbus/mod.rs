use crate::sansio::{Satisfy, Wants};
use connector::DBusConnector;
use libc::sockaddr_un;
use reader::DBusReader;
use writer::DBusWriter;

pub(crate) use queue::{DBusQueue, SessionDBusQueue, SystemDBusQueue};

mod connector;
mod queue;
mod reader;
mod writer;

pub(crate) struct DBusConnection {
    state: State,
    kind: DBusConnectionKind,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub(crate) enum DBusConnectionKind {
    System = 0,
    Session = 1,
}

enum State {
    Connecting(DBusConnector),
    Ready {
        reader: DBusReader,
        writer: DBusWriter,
    },
    Dead,
}

impl DBusConnection {
    pub(crate) fn new(addr: sockaddr_un, kind: DBusConnectionKind) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
            kind,
        }
    }

    pub(crate) fn dummy(kind: DBusConnectionKind) -> Self {
        Self {
            state: State::Dead,
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(),
            State::Ready { reader, writer } => match (reader.wants(), writer.wants()) {
                (
                    Wants::Read {
                        fd,
                        buf: readbuf,
                        len: readlen,
                    },
                    Wants::Write {
                        buf: writebuf,
                        len: writelen,
                        ..
                    },
                ) => Wants::ReadWrite {
                    fd,
                    readbuf,
                    readlen,
                    writebuf,
                    writelen,
                },

                (read, Wants::Nothing) => read,
                (Wants::Nothing, write) => write,
                other => unreachable!("DBus reader/write never want {other:?}"),
            },
            State::Dead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<&'static [u8]> {
        match &mut self.state {
            State::Connecting(connector) => match connector.satisfy(satisfy, res) {
                Ok(Some(fd)) => {
                    self.state = State::Ready {
                        reader: DBusReader::new(fd, self.kind),
                        writer: DBusWriter::new(fd, self.kind),
                    };
                }
                Ok(None) => {}
                Err(err) => {
                    log::error!("DBus {:?} crashed: {err:?}", self.kind);
                    self.stop();
                }
            },

            State::Ready { reader, writer } => match satisfy {
                Satisfy::Read => match reader.satisfy(satisfy, res) {
                    Ok(Some(buf)) => return Some(buf),
                    Ok(None) => {}
                    Err(err) => {
                        log::error!("DBus {:?} crashed: {err:?}", self.kind);
                        self.stop();
                    }
                },

                Satisfy::Write => {
                    if let Err(err) = writer.satisfy(satisfy, res) {
                        log::error!("DBus {:?} crashed: {err:?}", self.kind);
                        self.stop();
                    }
                }

                _ => {
                    log::error!(
                        "DBus {:?} in r/w mode received unexpected satisfy: {satisfy:?}",
                        self.kind
                    );
                    self.stop();
                }
            },

            State::Dead => {}
        }

        None
    }

    pub(crate) fn stop(&mut self) {
        self.state = State::Dead;
    }
}
