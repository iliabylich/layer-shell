use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{EnvHelper, NlSeparatedBuffer, SockaddrUn, StringRef, StringRefExt as _},
};
use libc::sockaddr_un;
use microjson::{JSONParsingError, JSONValue};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
enum State {
    Writer(UnixSocketOneshotWriter),
    Reader(UnixSocketReader),
}

pub struct Niri {
    state: State,
    layouts: FixedSizeArrray<10, StringRef>,
    emitter: Emitter,
}

impl Niri {
    pub(crate) fn address() -> Option<sockaddr_un> {
        let path = EnvHelper::niri_socket()?;
        let addr = SockaddrUn::from_bytes(path);
        Some(addr)
    }

    pub(crate) fn new(emitter: Emitter) -> Self {
        Self {
            state: State::Writer(UnixSocketOneshotWriter::new(b"\"EventStream\"\n")),
            layouts: FixedSizeArrray::empty_with_default_fn(StringRef::empty),
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
        let idx = match event {
            NiriEvent::KeyboardLayoutsChanged {
                layout_names,
                current_idx,
            } => {
                self.layouts = layout_names;
                current_idx
            }
            NiriEvent::KeyboardLayoutSwitched { idx } => idx,
        };

        let lang = self
            .layouts
            .get(idx)
            .ok_or(NiriError::NoLayoutIdx(idx))?
            .as_str();

        let lang = match lang {
            "English (US)" => "EN",
            "Polish" => "PL",
            _ => lang,
        };

        self.emitter.emit(&IoEvent::Language {
            lang: StringRef::new(lang),
        });

        Ok(())
    }
}

#[derive(Debug)]
pub enum NiriEvent {
    KeyboardLayoutsChanged {
        layout_names: FixedSizeArrray<10, StringRef>,
        current_idx: usize,
    },
    KeyboardLayoutSwitched {
        idx: usize,
    },
}

fn jsonerr(message: &'static str) -> impl Fn(JSONParsingError) -> NiriError {
    move |err: JSONParsingError| NiriError::JSONError { message, err }
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
            Err(JSONParsingError::KeyNotFound) => {}
            Err(err) => return Err(jsonerr("failed to get KeyboardLayoutsChanged")(err)),
        }

        match json.get_key_value("KeyboardLayoutSwitched") {
            Ok(json) => {
                let event = Self::parse_keyword_layout_switched(&json)?;
                return Ok(Some(event));
            }
            Err(JSONParsingError::KeyNotFound) => {}
            Err(err) => return Err(jsonerr("failed to get KeyboardLayoutSwitched")(err)),
        }

        Ok(None)
    }

    fn parse_keyboard_layouts_changed(json: &JSONValue) -> Result<Self, NiriError> {
        let keyboard_layouts = json
            .get_key_value("keyboard_layouts")
            .map_err(jsonerr("failed to get keyboard_layouts"))?;
        let names = keyboard_layouts
            .get_key_value("names")
            .map_err(jsonerr("failed to get names"))?
            .iter_array()
            .map_err(jsonerr("names is not an array"))?;

        let mut layout_names = FixedSizeArrray::empty_with_default_fn(StringRef::empty);
        for name in names {
            let name = name
                .read_string()
                .map_err(jsonerr("failed to get layout name as a string"))?;
            layout_names
                .push(StringRef::new(name))
                .ok_or(NiriError::TooManyLayouts)?;
        }

        let current_idx = keyboard_layouts
            .get_key_value("current_idx")
            .map_err(jsonerr("failed to get current_idx"))?
            .read_integer()
            .map_err(jsonerr("current_idx is not an integer"))?;
        let current_idx = usize::try_from(current_idx).map_err(NiriError::NegativeCurrentIdx)?;
        Ok(Self::KeyboardLayoutsChanged {
            layout_names,
            current_idx,
        })
    }

    fn parse_keyword_layout_switched(json: &JSONValue) -> Result<Self, NiriError> {
        let idx = json
            .get_key_value("idx")
            .map_err(jsonerr("failed to get idx"))?
            .read_integer()
            .map_err(jsonerr("idx is not an integer"))?;
        let idx = usize::try_from(idx).map_err(NiriError::NegativeIdx)?;
        Ok(Self::KeyboardLayoutSwitched { idx })
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum NiriError {
    #[error("no such layout index: {0}")]
    NoLayoutIdx(usize),
    #[error("non-utf8 niri json")]
    NonUtf8Json(core::str::Utf8Error),

    #[error("JSON error: {message}, {err}")]
    JSONError {
        message: &'static str,
        err: JSONParsingError,
    },

    #[error("too many layouts")]
    TooManyLayouts,

    #[error("current_idx is negative")]
    NegativeCurrentIdx(core::num::TryFromIntError),
    #[error("idx is negative")]
    NegativeIdx(core::num::TryFromIntError),
}
