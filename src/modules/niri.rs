use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    module_id::ModuleId,
    modules::Module,
    utils::{EnvHelper, NlSeparatedBuffer, StringRef, StringRefExt, unix_socket, unix_socket_addr},
};
use microjson::{JSONParsingError, JSONValue};
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};

pub struct Niri {
    fd: OwnedFd,
    buf: NlSeparatedBuffer,
    layouts: FixedSizeArrray<10, StringRef>,
}

impl Niri {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating Niri");

        let Some(path) = EnvHelper::niri_socket() else {
            log::error!("no $NIRI_SOCKET");
            return None;
        };
        let addr = unix_socket_addr!("{path}").ok()?;
        let fd = unix_socket!().ok()?;

        if let Err(err) = rustix::net::connect(&fd, &addr) {
            log::error!("failed to connect(): {err:?}");
            return None;
        }

        let buf = b"\"EventStream\"\n";
        match rustix::io::write(&fd, buf) {
            Ok(len) if buf.len() == len => {}
            Ok(len) => {
                log::error!("failed to write initial message: written {len}");
                return None;
            }
            Err(err) => {
                log::error!("failed to write: {err:?}");
                return None;
            }
        }

        Some(Self {
            fd,
            buf: NlSeparatedBuffer::new(),
            layouts: FixedSizeArrray::empty_with_default_fn(StringRef::empty),
        })
    }

    fn process(&mut self, event: NiriEvent, emitter: Emitter) -> Result<(), ()> {
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

        emitter.emit(&IoEvent::Language {
            lang: StringRef::new(lang),
        });

        Ok(())
    }
}

impl Module for Niri {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let count = rustix::io::read(&self.fd, self.buf.remainder())
            .map_err(|err| log::error!("failed to read(): {err:?}"))?;

        self.buf.written(count);
        while let Some(bytes) = self.buf.pre_nl() {
            if let Some(event) = NiriEvent::from_json(bytes)? {
                self.process(event, emitter)?;
            }
            self.buf.drop_pre_nl();
        }

        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::Niri
    }

    const MODULE_ID: ModuleId = ModuleId::Niri;
}

impl AsFd for Niri {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
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
