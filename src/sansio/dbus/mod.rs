use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail};
use connector::DBusConnector;
use libc::sockaddr_un;
use reader::DBusReader;
use writer::DBusWriter;

pub(crate) use queue::DBusQueue;

mod connector;
mod queue;
mod reader;
mod writer;

pub(crate) struct DBusConnection {
    state: State,
    queue: DBusQueue,
}

enum State {
    Connecting(DBusConnector),
    Ready {
        reader: DBusReader,
        writer: DBusWriter,
    },
}

impl DBusConnection {
    pub(crate) fn new(addr: sockaddr_un, queue: DBusQueue) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
            queue,
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
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        match (&mut self.state, satisfy) {
            (State::Connecting(connector), _) => {
                if let Some(fd) = connector.satisfy(satisfy, res)? {
                    self.state = State::Ready {
                        reader: DBusReader::new(fd),
                        writer: DBusWriter::new(fd, self.queue.clone()),
                    };
                }
                Ok(None)
            }

            (State::Ready { reader, .. }, Satisfy::Read) => {
                if let Some(buf) = reader.satisfy(satisfy, res)? {
                    Ok(Some(buf))
                } else {
                    Ok(None)
                }
            }

            (State::Ready { writer, .. }, Satisfy::Write) => {
                writer.satisfy(satisfy, res)?;
                Ok(None)
            }

            (_, satisfy) => {
                bail!("malformed DBusReader state: reader/writer can't handle {satisfy:?}")
            }
        }
    }
}
