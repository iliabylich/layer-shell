use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{StringRef, StringRefExt as _, getenv, new_sockaddr_un},
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};
use anyhow::{Context, Result};
use buffer::{Buffer, NiriEvent};
use libc::sockaddr_un;

mod buffer;

enum State {
    Writer(UnixSocketOneshotWriter),
    Reader(Box<UnixSocketReader>),
}

pub(crate) struct Niri {
    state: State,
    buffer: Buffer,
    layouts: Vec<String>,
    emitter: Emitter,
}

impl Niri {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let path = getenv(c"NIRI_SOCKET").context("no $NIRI_SOCKET")?;
        let addr = new_sockaddr_un(path)?;
        Ok(addr)
    }

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            state: State::Writer(UnixSocketOneshotWriter::new(b"\"EventStream\"\n")),
            buffer: Buffer::new(),
            layouts: vec![],
            emitter,
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        match &mut self.state {
            State::Writer(writer) => writer.wants(addr),
            State::Reader(reader) => reader.wants(addr),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<()> {
        match &mut self.state {
            State::Writer(writer) => {
                if let Some(fd) = writer.satisfy(satisfy)? {
                    self.state =
                        State::Reader(Box::new(UnixSocketReader::new_connected_from_fd(fd)));
                }
            }

            State::Reader(reader) => {
                let Some((buf, len)) = reader.satisfy(satisfy)? else {
                    return Ok(());
                };
                let buf = buf.get(..len).context("buf is too short")?;
                self.process(buf)?;
            }
        }

        Ok(())
    }

    fn process(&mut self, buf: &[u8]) -> Result<()> {
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

            self.emitter.emit(&Event::Language {
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}
