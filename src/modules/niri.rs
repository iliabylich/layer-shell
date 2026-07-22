use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    utils::{EnvHelper, NlSeparatedBuffer, SockaddrUn, StringRef, StringRefExt},
};
use libc::sockaddr_un;
use microjson::{JSONParsingError, JSONValue};

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
        log::trace!("Creating Niri");

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
        let wants = match &mut self.state {
            State::Writer(writer) => writer.wants(addr)?,
            State::Reader(reader) => reader.wants(addr, buf.remainder())?,
        };
        log::trace!("{wants:?}");
        Some(wants)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut NlSeparatedBuffer,
    ) -> Result<(), ()> {
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

    fn process(&mut self, event: NiriEvent) -> Result<(), ()> {
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
            .ok_or_else(|| {
                log::error!("no layout with idx {idx}: {:?}", self.layouts);
            })?
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

impl NiriEvent {
    pub(crate) fn from_json(bytes: &[u8]) -> Result<Option<Self>, ()> {
        let s = core::str::from_utf8(bytes).map_err(|err| {
            log::error!("non-utf8 source: {err:?}");
        })?;
        let json = JSONValue::load(s);

        match json.get_key_value("KeyboardLayoutsChanged") {
            Ok(json) => {
                let event = Self::parse_keyboard_layouts_changed(&json)?;
                return Ok(Some(event));
            }
            Err(JSONParsingError::KeyNotFound) => {}
            Err(err) => {
                log::error!("failed to get KeyboardLayoutsChanged: {err:?}");
                return Err(());
            }
        }

        match json.get_key_value("KeyboardLayoutSwitched") {
            Ok(json) => {
                let event = Self::parse_keyword_layout_switched(&json)?;
                return Ok(Some(event));
            }
            Err(JSONParsingError::KeyNotFound) => {}
            Err(err) => {
                log::error!("failed to get KeyboardLayoutSwitched: {err:?}");
                return Err(());
            }
        }

        Ok(None)
    }

    fn parse_keyboard_layouts_changed(json: &JSONValue) -> Result<Self, ()> {
        let keyboard_layouts = json.get_key_value("keyboard_layouts").map_err(|err| {
            log::error!("failed to get keyboard_layouts: {err:?}");
        })?;
        let names = keyboard_layouts
            .get_key_value("names")
            .map_err(|err| {
                log::error!("failed to get names: {err:?}");
            })?
            .iter_array()
            .map_err(|err| {
                log::error!("names is not an array: {err:?}");
            })?;

        let mut layout_names = FixedSizeArrray::empty_with_default_fn(StringRef::empty);
        for name in names {
            let name = name.read_string().map_err(|err| {
                log::error!("failed to get layout name as a string: {err:?}");
            })?;
            layout_names.push(StringRef::new(name)).ok_or_else(|| {
                log::error!("too many layouts");
            })?;
        }

        let current_idx = keyboard_layouts
            .get_key_value("current_idx")
            .map_err(|err| {
                log::error!("failed to get current_idx: {err:?}");
            })?
            .read_integer()
            .map_err(|err| {
                log::error!("current_idx is not an integer: {err:?}");
            })?;
        let current_idx = usize::try_from(current_idx).map_err(|err| {
            log::error!("negative current idx: {err:?}");
        })?;
        Ok(Self::KeyboardLayoutsChanged {
            layout_names,
            current_idx,
        })
    }

    fn parse_keyword_layout_switched(json: &JSONValue) -> Result<Self, ()> {
        let idx = json
            .get_key_value("idx")
            .map_err(|err| {
                log::error!("failed to get idx: {err:?}");
            })?
            .read_integer()
            .map_err(|err| {
                log::error!("idx is not an integer: {err:?}");
            })?;
        let idx = usize::try_from(idx).map_err(|err| {
            log::error!("negative idx: {err:?}");
        })?;
        Ok(Self::KeyboardLayoutSwitched { idx })
    }
}
