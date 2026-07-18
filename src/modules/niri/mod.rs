use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{StringRef, StringRefExt as _, getenv, new_sockaddr_un},
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use anyhow::{Context, Result, bail};
use buffer::{Buffer, NiriEvent};
use libc::sockaddr_un;

mod buffer;

enum State {
    Writer(Box<UnixSocketOneshotWriter>),
    Reader(Box<UnixSocketReader>),
}

pub(crate) struct Niri {
    state: State,
    buffer: Buffer,
    layouts: Vec<String>,
}

impl Niri {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let path = getenv(c"NIRI_SOCKET").context("no $NIRI_SOCKET")?;
        let addr = new_sockaddr_un(path)?;
        Ok(addr)
    }

    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            state: State::Writer(Box::new(UnixSocketOneshotWriter::new("\"EventStream\"\n")?)),
            buffer: Buffer::new(),
            layouts: vec![],
        })
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        match &mut self.state {
            State::Writer(writer) => writer.wants(addr),
            State::Reader(reader) => reader.wants(addr),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        match &mut self.state {
            State::Writer(writer) => match satisfy {
                Satisfy::Socket(res) => {
                    let fd = res?;
                    writer.satisfy_socket(fd)?;
                }

                Satisfy::Connect(res) => {
                    res?;
                    writer.satisfy_connect()?;
                }

                Satisfy::Write(res) => {
                    let _ = res?;
                    writer.satisfy_write()?;
                    self.state = State::Reader(Box::new(UnixSocketReader::new_connected_from_fd(
                        writer.fd()?,
                    )));
                }

                _ => bail!("Niri writer only accepts Socket, Connect and Write, got: {satisfy:?}"),
            },

            State::Reader(reader) => match satisfy {
                Satisfy::Socket(res) => {
                    let fd = res?;
                    reader.satisfy_socket(fd)?;
                }

                Satisfy::Connect(res) => {
                    res?;
                    reader.satisfy_connect()?;
                }

                Satisfy::Read(res) => {
                    let bytes_read = res?;
                    let (buf, len) = reader.satisfy_read(bytes_read)?;
                    let buf = buf.get(..len).context("buf is too short")?;
                    self.process(buf, events)?;
                }

                _ => bail!("Niri reader only accepts Socket, Connect and Read, got: {satisfy:?}"),
            },
        }

        Ok(())
    }

    fn process(&mut self, buf: &[u8], events: &mut EventQueue) -> Result<()> {
        let niri_events = self.buffer.push(buf)?;
        let mut layouts = None;
        let mut current_layout_idx = None;

        for event in niri_events {
            match event {
                NiriEvent::KeyboardLayoutsChanged { keyboard_layouts } => {
                    layouts = Some(keyboard_layouts.names);
                    current_layout_idx = Some(keyboard_layouts.current_idx);
                }
                NiriEvent::KeyboardLayoutSwitched { idx } => {
                    current_layout_idx = Some(idx);
                }
            }
        }

        if let Some(layouts) = layouts {
            self.layouts = layouts;
        }
        if let Some(current_layout_idx) = current_layout_idx {
            let mut lang = self
                .layouts
                .get(current_layout_idx)
                .context("no such layout idx")?
                .as_str();

            if lang == "English (US)" {
                lang = "EN";
            } else if lang == "Polish" {
                lang = "PL";
            } else {
                lang = "??";
            }

            events.push_back(Event::Language {
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}
