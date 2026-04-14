use crate::sansio::{Satisfy, Wants};
use connector::DBusConnector;
use libc::sockaddr_un;
use reader::DBusReader;
use writer::DBusWriter;

pub(crate) use kind::DBusConnectionKind;
pub(crate) use queue::{DBusQueue, SessionDBusQueue, SystemDBusQueue};

mod connector;
mod kind;
mod queue;
mod reader;
mod writer;

pub(crate) struct DBusConnection {
    state: State,
    kind: DBusConnectionKind,
}

enum State {
    Connecting(DBusConnector),
    Ready {
        reader: DBusReader,
        writer: DBusWriter,
    },
}

impl DBusConnection {
    pub(crate) fn new(addr: sockaddr_un, kind: DBusConnectionKind) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr, kind)),
            kind,
        }
    }

    pub(crate) fn dummy(kind: DBusConnectionKind) -> Self {
        Self {
            state: State::Connecting(DBusConnector::dummy(kind)),
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(),
            State::Ready { reader, writer } => match (reader.wants(), writer.wants()) {
                (
                    Some(Wants::Read {
                        fd,
                        buf: readbuf,
                        len: readlen,
                    }),
                    Some(Wants::Write {
                        buf: writebuf,
                        len: writelen,
                        ..
                    }),
                ) => Some(Wants::ReadWrite {
                    fd,
                    readbuf,
                    readlen,
                    writebuf,
                    writelen,
                }),

                (read, None) => read,
                (None, write) => write,
                other => {
                    log::error!("bug: DBus reader/write never want {other:?}");
                    None
                }
            },
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<&'static [u8]> {
        match &mut self.state {
            State::Connecting(connector) => {
                let fd = connector.satisfy(satisfy, res)?;
                self.state = State::Ready {
                    reader: DBusReader::new(fd, self.kind),
                    writer: DBusWriter::new(fd, self.kind),
                };
            }

            State::Ready { reader, writer } => match satisfy {
                Satisfy::Read => return reader.satisfy(satisfy, res),

                Satisfy::Write => writer.satisfy(satisfy, res),

                _ => {
                    log::error!(
                        "DBus {:?} in r/w mode received unexpected satisfy: {satisfy:?}",
                        self.kind
                    );
                    reader.stop();
                    writer.stop();
                }
            },
        }

        None
    }

    pub(crate) fn stop(&mut self) {
        match &mut self.state {
            State::Connecting(connector) => connector.stop(),
            State::Ready { reader, writer } => {
                reader.stop();
                writer.stop();
            }
        }
    }
}
