use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{NlSeparatedBuffer, StringRef, StringRefExt as _, getenv, new_sockaddr_un},
};
use libc::sockaddr_un;
use microjson::JSONValue;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
enum State {
    Writer(UnixSocketOneshotWriter),
    Reader(UnixSocketReader),
}

pub(crate) struct Niri {
    state: State,
    layouts: FixedSizeArrray<10, StringRef>,
    emitter: Emitter,
}

impl Niri {
    pub(crate) fn address() -> Result<sockaddr_un, NiriError> {
        let path = getenv(c"NIRI_SOCKET").ok_or(NiriError::NoNiriSocket)?;
        let addr = new_sockaddr_un(path);
        Ok(addr)
    }

    pub(crate) fn new(emitter: Emitter) -> Self {
        Self {
            state: State::Writer(UnixSocketOneshotWriter::new(b"\"EventStream\"\n")),
            layouts: FixedSizeArrray::empty_with_default_fn(|| StringRef::new("")),
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

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut NlSeparatedBuffer,
    ) -> Result<(), IoError> {
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

    fn process(&mut self, event: NiriEvent) -> Result<(), NiriError> {
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
        if let Some(idx) = current_layout_idx {
            let mut lang = self
                .layouts
                .get(idx)
                .ok_or(NiriError::NoLayoutIdx { idx })?
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
        layout_names: FixedSizeArrray<10, StringRef>,
        current_idx: usize,
    },
    KeyboardLayoutSwitched {
        idx: usize,
    },
}

impl NiriEvent {
    pub(crate) fn from_json(bytes: &[u8]) -> Result<Option<Self>, NiriError> {
        let s = core::str::from_utf8(bytes).map_err(NiriError::NonUtf8Json)?;
        let json = JSONValue::load(s);

        match json.get_key_value("KeyboardLayoutsChanged") {
            Ok(json) => {
                let event = Self::parse_keyboard_layouts_changed(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => {
                    return Err(NiriError::JSONError {
                        ctx: "failed to get KeyboardLayoutsChanged",
                        err,
                    });
                }
            },
        }

        match json.get_key_value("KeyboardLayoutSwitched") {
            Ok(json) => {
                let event = Self::parse_keyword_layout_switched(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => {
                    return Err(NiriError::JSONError {
                        ctx: "failed to get KeyboardLayoutSwitched",
                        err,
                    });
                }
            },
        }

        Ok(None)
    }

    fn parse_keyboard_layouts_changed(json: &JSONValue) -> Result<Self, NiriError> {
        let keyboard_layouts =
            json.get_key_value("keyboard_layouts")
                .map_err(|err| NiriError::JSONError {
                    ctx: "failed to get keyboard_layouts",
                    err,
                })?;
        let names = keyboard_layouts
            .get_key_value("names")
            .map_err(|err| NiriError::JSONError {
                ctx: "failed to get names",
                err,
            })?
            .iter_array()
            .map_err(|err| NiriError::JSONError {
                ctx: "names is not an array",
                err,
            })?;

        let mut layout_names = FixedSizeArrray::empty_with_default_fn(|| StringRef::new(""));
        for name in names {
            let name = name.read_string().map_err(|err| NiriError::JSONError {
                ctx: "failed to get layout name as a string",
                err,
            })?;
            layout_names
                .push(StringRef::new(name))
                .ok_or(NiriError::TooManyLayouts)?;
        }

        let current_idx = keyboard_layouts
            .get_key_value("current_idx")
            .map_err(|err| NiriError::JSONError {
                ctx: "failed to get current_idx",
                err,
            })?
            .read_integer()
            .map_err(|err| NiriError::JSONError {
                ctx: "current_idx is not an integer",
                err,
            })?;
        let current_idx = usize::try_from(current_idx).map_err(NiriError::NegativeCurrentIdx)?;
        Ok(Self::KeyboardLayoutsChanged {
            layout_names,
            current_idx,
        })
    }

    fn parse_keyword_layout_switched(json: &JSONValue) -> Result<Self, NiriError> {
        let idx = json
            .get_key_value("idx")
            .map_err(|err| NiriError::JSONError {
                ctx: "failed to get idx",
                err,
            })?
            .read_integer()
            .map_err(|err| NiriError::JSONError {
                ctx: "idx is not an integer",
                err,
            })?;
        let idx = usize::try_from(idx).map_err(NiriError::NegativeIdx)?;
        Ok(Self::KeyboardLayoutSwitched { idx })
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub(crate) enum NiriError {
    #[error("$NIRI_SOCKET is not set")]
    NoNiriSocket,
    #[error("no such layout index: {idx}")]
    NoLayoutIdx { idx: usize },
    #[error("non-utf8 niri json")]
    NonUtf8Json(core::str::Utf8Error),

    #[error("JSON error: {ctx}, {err}")]
    JSONError {
        ctx: &'static str,
        err: microjson::JSONParsingError,
    },

    #[error("too many layouts")]
    TooManyLayouts,

    #[error("current_idx is negative")]
    NegativeCurrentIdx(core::num::TryFromIntError),
    #[error("idx is negative")]
    NegativeIdx(core::num::TryFromIntError),
}
