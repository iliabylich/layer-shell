use crate::{
    IoEvent,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{NlSeparatedBuffer, StringRef, StringRefExt as _, get_json, getenv, new_sockaddr_un},
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use anyhow::{Context, Result};
use libc::sockaddr_un;
use microjson::JSONValue;

#[derive(Debug, Clone, Copy)]
enum State {
    Writer(UnixSocketOneshotWriter),
    Reader(UnixSocketReader),
}

pub(crate) struct Niri {
    state: State,
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
            layouts: vec![],
            emitter,
        }
    }

    pub(crate) fn wants(
        &mut self,
        addr: &sockaddr_un,
        buf: &mut NlSeparatedBuffer,
    ) -> Option<Wants> {
        match &mut self.state {
            State::Writer(writer) => writer.wants(addr),
            State::Reader(reader) => reader.wants(addr, buf.remainder()),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: &mut NlSeparatedBuffer) -> Result<()> {
        match &mut self.state {
            State::Writer(writer) => {
                if let Some(fd) = writer.satisfy(satisfy)? {
                    self.state = State::Reader(UnixSocketReader::new_connected_from_fd(fd));
                }
            }

            State::Reader(reader) => {
                if let Some(written) = reader.satisfy(satisfy)? {
                    buf.written(written);
                    while let Some(bytes) = buf.pre_nl() {
                        if let Some(event) = NiriEvent::from_json(bytes)? {
                            self.process(event)?;
                        }
                        buf.drop_pre_nl();
                    }
                }
            }
        }

        Ok(())
    }

    fn process(&mut self, event: NiriEvent) -> Result<()> {
        let mut layouts = None;
        let current_layout_idx;

        match event {
            NiriEvent::KeyboardLayoutsChanged {
                layout_names,
                current_idx,
            } => {
                layouts = Some(layout_names);
                current_layout_idx = Some(current_idx);
            }
            NiriEvent::KeyboardLayoutSwitched { idx } => {
                current_layout_idx = Some(idx);
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

            self.emitter.emit(&IoEvent::Language {
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum NiriEvent {
    KeyboardLayoutsChanged {
        layout_names: Vec<String>,
        current_idx: usize,
    },
    KeyboardLayoutSwitched {
        idx: usize,
    },
}

impl NiriEvent {
    pub(crate) fn from_json(bytes: &[u8]) -> Result<Option<Self>> {
        let s = core::str::from_utf8(bytes)?;
        let json = JSONValue::load(s);

        match json.get_key_value("KeyboardLayoutsChanged") {
            Ok(json) => {
                let event = Self::parse_keyboard_layouts_changed(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => return Err(anyhow::anyhow!(err)),
            },
        }

        match json.get_key_value("KeyboardLayoutSwitched") {
            Ok(json) => {
                let event = Self::parse_keyword_layout_switched(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => return Err(anyhow::anyhow!(err)),
            },
        }

        Ok(None)
    }

    fn parse_keyboard_layouts_changed(json: &JSONValue) -> Result<Self> {
        let keyboard_layouts = json
            .get_key_value("keyboard_layouts")
            .map_err(|err| anyhow::anyhow!(err))?;
        let names = keyboard_layouts
            .get_key_value("names")
            .map_err(|err| anyhow::anyhow!(err))?
            .iter_array()
            .map_err(|err| anyhow::anyhow!(err))?
            .map(|name| {
                name.read_string()
                    .map(ToString::to_string)
                    .map_err(|err| anyhow::anyhow!(err))
            })
            .collect::<Result<Vec<_>>>()?;

        let current_idx = get_json!(keyboard_layouts, "current_idx", read_integer);
        let current_idx = usize::try_from(current_idx).context("negative keyboard current_idx")?;
        Ok(Self::KeyboardLayoutsChanged {
            layout_names: names,
            current_idx,
        })
    }

    fn parse_keyword_layout_switched(json: &JSONValue) -> Result<Self> {
        let idx = get_json!(json, "idx", read_integer);
        let idx = usize::try_from(idx).context("negative keyboard idx")?;
        Ok(Self::KeyboardLayoutSwitched { idx })
    }
}
