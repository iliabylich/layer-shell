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

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        if let State::Connecting(connector) = &mut self.state {
            if let Some(fd) = connector.satisfy(satisfy, res)? {
                self.state = State::Ready {
                    reader: DBusReader::new(fd),
                    writer: DBusWriter::new(fd, self.queue.clone()),
                };
            }
            return Ok(None);
        }

        if let State::Ready { reader, writer } = &mut self.state
            && matches!(satisfy, Satisfy::Read | Satisfy::Write)
        {
            match satisfy {
                Satisfy::Read => {
                    if let Some(buf) = reader.satisfy(satisfy, res)? {
                        return Ok(Some(buf));
                    } else {
                        return Ok(None);
                    }
                }

                Satisfy::Write => {
                    writer.satisfy(satisfy, res)?;
                    return Ok(None);
                }

                _ => unreachable!(),
            }
        }

        bail!("malformed DBusReader state: reader/writer can't handle {satisfy:?}")
    }
}
