use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{StringRef, StringRefExt, getenv, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use libc::sockaddr_un;

pub(crate) struct Tray {
    reader: Box<UnixSocketReader>,
    buf: Buffer,
    writebuf: [u8; 8],
}

impl Tray {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/tray-mon.sock");
        let addr = new_sockaddr_un(path.as_bytes())?;
        Ok(addr)
    }

    pub(crate) fn new() -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            buf: Buffer::new(),
            writebuf: [0; _],
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        self.reader.wants(addr)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        match satisfy {
            Satisfy::Socket(res) => {
                let fd = res?;
                self.reader.satisfy_socket(fd)?;
                Ok(())
            }

            Satisfy::Connect(res) => {
                res?;
                self.reader.satisfy_connect()?;
                Ok(())
            }

            Satisfy::Write(res) => {
                res?;
                Ok(())
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let (buf, len) = self.reader.satisfy_read(bytes_read)?;
                let bytes = buf.get(..len).context("buf is too short")?;

                for event in self.buf.push(bytes) {
                    let event = match event {
                        TrayEvent::AppAdded {
                            service,
                            icon,
                            menu,
                        } => Event::TrayAppAdded {
                            service,
                            menu,
                            icon: StringRef::new(icon.as_str()?),
                        },
                        TrayEvent::AppRemoved { service } => Event::TrayAppRemoved { service },
                        TrayEvent::MenuUpdated { service, menu } => {
                            Event::TrayAppMenuUpdated { service, menu }
                        }
                        TrayEvent::IconUpdated { service, icon } => Event::TrayAppIconUpdated {
                            service,
                            icon: StringRef::new(icon.as_str()?),
                        },
                    };

                    events.push_back(event);
                }

                Ok(())
            }

            _ => bail!("KbMod only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }

    pub(crate) fn wants_trigger(&mut self, service: u32, id: i32) -> Option<Wants> {
        self.writebuf[0..4].copy_from_slice(&service.to_be_bytes());
        self.writebuf[4..8].copy_from_slice(&id.to_be_bytes());
        Some(Wants::Write {
            fd: self.reader.fd()?,
            buf: self.writebuf.as_ptr(),
            len: self.writebuf.len(),
        })
    }
}

struct Buffer(Vec<u8>);
impl Buffer {
    const fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, bytes: &[u8]) -> Vec<TrayEvent> {
        self.0.extend_from_slice(bytes);
        let mut events = vec![];

        while let Some((first, rest)) = self
            .0
            .split_first_chunk::<{ TrayEvent::SERIALIZED_BYTESIZE }>()
        {
            let Some(event) = TrayEvent::deserialize(first) else {
                log::error!("failed to deserialize event");
                continue;
            };
            events.push(event);
            self.0 = rest.to_vec();
        }

        events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
#[expect(clippy::large_enum_variant)]
pub(crate) enum TrayEvent {
    AppAdded {
        service: u32,
        icon: TrayFixedSizeString,
        menu: TrayMenu,
    },
    AppRemoved {
        service: u32,
    },
    MenuUpdated {
        service: u32,
        menu: TrayMenu,
    },
    IconUpdated {
        service: u32,
        icon: TrayFixedSizeString,
    },
}
impl TrayEvent {
    pub(crate) const SERIALIZED_BYTESIZE: usize = 1
        + size_of::<u32>()
        + TrayFixedSizeString::SERIALIZED_BYTESIZE
        + TrayMenu::SERIALIZED_BYTESIZE;

    pub(crate) fn deserialize(buf: &[u8; Self::SERIALIZED_BYTESIZE]) -> Option<Self> {
        let mut buf = &buf[..];

        match read_u8(&mut buf)? {
            1 => {
                let service = read_u32(&mut buf)?;
                let icon = read_fixed_size_string(&mut buf)?;
                let menu = read_menu(&mut buf)?;
                Some(Self::AppAdded {
                    service,
                    icon,
                    menu,
                })
            }
            2 => {
                let service = read_u32(&mut buf)?;
                Some(Self::AppRemoved { service })
            }
            3 => {
                let service = read_u32(&mut buf)?;
                let menu = read_menu(&mut buf)?;
                Some(Self::MenuUpdated { service, menu })
            }
            4 => {
                let service = read_u32(&mut buf)?;
                let icon = read_fixed_size_string(&mut buf)?;
                Some(Self::IconUpdated { service, icon })
            }
            _ => None,
        }
    }
}

pub const TRAY_MENU_ITEMS_COUNT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct TrayMenu(pub [MaybeRootTrayElement; TRAY_MENU_ITEMS_COUNT]);

impl TrayMenu {
    pub(crate) const SERIALIZED_BYTESIZE: usize =
        TRAY_MENU_ITEMS_COUNT * TrayElement::SERIALIZED_BYTESIZE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
#[repr(C)]
pub struct MaybeRootTrayElement {
    root: bool,
    element: TrayElement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
#[repr(C)]
pub enum TrayElement {
    Regular {
        id: i32,
        label: TrayLabel,
    },
    Disabled {
        id: i32,
        label: TrayLabel,
    },
    Checkbox {
        id: i32,
        label: TrayLabel,
        checked: bool,
    },
    Radio {
        id: i32,
        label: TrayLabel,
        selected: bool,
    },
    Nested {
        id: i32,
        label: TrayLabel,
        child_start_idx: u64,
        child_end_idx: u64,
    },
    Section {
        child_start_idx: u64,
        child_end_idx: u64,
    },
    None,
}
impl TrayElement {
    pub const SERIALIZED_BYTESIZE: usize = size_of::<u8>()
        + size_of::<i32>()
        + size_of::<u8>()
        + TrayLabel::SERIALIZED_BYTESIZE
        + size_of::<u64>() * 2;

    pub fn deserialize(buf: [u8; Self::SERIALIZED_BYTESIZE]) -> Option<(Self, bool)> {
        let mut buf = &buf[..];

        match read_u8(&mut buf)? {
            0 => Some((Self::None, false)),

            1 => {
                let id = read_i32(&mut buf)?;
                let root = read_u8(&mut buf)? == 1;
                let label = read_label(&mut buf)?;
                Some((Self::Regular { id, label }, root))
            }

            2 => {
                let id = read_i32(&mut buf)?;
                let root = read_u8(&mut buf)? == 1;
                let label = read_label(&mut buf)?;
                Some((Self::Disabled { id, label }, root))
            }

            3 => {
                let id = read_i32(&mut buf)?;
                let root = read_u8(&mut buf)? == 1;
                let checked = read_u8(&mut buf)?;
                let label = read_label(&mut buf)?;
                Some((
                    Self::Checkbox {
                        id,
                        label,
                        checked: checked == 1,
                    },
                    root,
                ))
            }

            4 => {
                let id = read_i32(&mut buf)?;
                let root = read_u8(&mut buf)? == 1;
                let selected = read_u8(&mut buf)?;
                let label = read_label(&mut buf)?;
                Some((
                    Self::Radio {
                        id,
                        label,
                        selected: selected == 1,
                    },
                    root,
                ))
            }

            5 => {
                let id = read_i32(&mut buf)?;
                let root = read_u8(&mut buf)? == 1;
                let child_start_idx = read_u64(&mut buf)?;
                let child_end_idx = read_u64(&mut buf)?;
                let label = read_label(&mut buf)?;
                Some((
                    Self::Nested {
                        id,
                        label,
                        child_start_idx,
                        child_end_idx,
                    },
                    root,
                ))
            }

            6 => {
                let root = read_u8(&mut buf)? == 1;
                let child_start_idx = read_u64(&mut buf)?;
                let child_end_idx = read_u64(&mut buf)?;
                Some((
                    Self::Section {
                        child_start_idx,
                        child_end_idx,
                    },
                    root,
                ))
            }

            _ => None,
        }
    }
}

pub const TRAY_LABEL_BYTESIZE: usize = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
#[must_use]
#[repr(C)]
pub struct TrayLabel {
    buf: [u8; TRAY_LABEL_BYTESIZE],
    len: u32,
}
impl TrayLabel {
    pub(crate) const SERIALIZED_BYTESIZE: usize = TRAY_LABEL_BYTESIZE + 4;

    pub(crate) fn deserialize(buf: [u8; Self::SERIALIZED_BYTESIZE]) -> Self {
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let mut s = [0; _];
        s.copy_from_slice(&buf[4..]);
        Self { len, buf: s }
    }

    pub(crate) fn as_bytes(&self) -> Result<&[u8]> {
        self.buf
            .get(..self.len as usize)
            .context("malformed TrayFixedSizeString")
    }

    pub(crate) fn as_str(&self) -> Result<&str> {
        let s = core::str::from_utf8(self.as_bytes()?)?;
        Ok(s)
    }
}
impl core::fmt::Debug for TrayLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[must_use]
pub(crate) struct TrayFixedSizeString {
    buf: [u8; Self::STR_BYTESIZE],
    len: u32,
}
impl TrayFixedSizeString {
    pub(crate) const STR_BYTESIZE: usize = 256;
    pub(crate) const SERIALIZED_BYTESIZE: usize = Self::STR_BYTESIZE + 4;

    pub(crate) fn deserialize(buf: &[u8; Self::SERIALIZED_BYTESIZE]) -> Self {
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let mut s = [0; _];
        s.copy_from_slice(&buf[4..]);
        Self { len, buf: s }
    }

    pub(crate) fn as_bytes(&self) -> Result<&[u8]> {
        self.buf
            .get(..self.len as usize)
            .context("malformed TrayFixedSizeString")
    }

    pub(crate) fn as_str(&self) -> Result<&str> {
        let s = core::str::from_utf8(self.as_bytes()?)?;
        Ok(s)
    }
}
impl core::fmt::Debug for TrayFixedSizeString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

fn read_u8(buf: &mut &[u8]) -> Option<u8> {
    let n = *buf.first()?;
    *buf = buf.get(1..)?;
    Some(n)
}
fn read_i32(buf: &mut &[u8]) -> Option<i32> {
    let n = i32::from_be_bytes([*buf.first()?, *buf.get(1)?, *buf.get(2)?, *buf.get(3)?]);
    *buf = buf.get(4..)?;
    Some(n)
}
fn read_u32(buf: &mut &[u8]) -> Option<u32> {
    let n = u32::from_be_bytes([*buf.first()?, *buf.get(1)?, *buf.get(2)?, *buf.get(3)?]);
    *buf = buf.get(4..)?;
    Some(n)
}
fn read_label(buf: &mut &[u8]) -> Option<TrayLabel> {
    let mut textbuf = [0; _];
    textbuf.copy_from_slice(buf.get(0..TrayLabel::SERIALIZED_BYTESIZE)?);
    let text = TrayLabel::deserialize(textbuf);
    *buf = buf.get(TrayLabel::SERIALIZED_BYTESIZE..)?;
    Some(text)
}
fn read_fixed_size_string(buf: &mut &[u8]) -> Option<TrayFixedSizeString> {
    let mut textbuf = [0; _];
    textbuf.copy_from_slice(buf.get(0..TrayFixedSizeString::SERIALIZED_BYTESIZE)?);
    let text = TrayFixedSizeString::deserialize(&textbuf);
    *buf = buf.get(TrayFixedSizeString::SERIALIZED_BYTESIZE..)?;
    Some(text)
}
fn read_u64(buf: &mut &[u8]) -> Option<u64> {
    let n = u64::from_be_bytes([
        *buf.first()?,
        *buf.get(1)?,
        *buf.get(2)?,
        *buf.get(3)?,
        *buf.get(4)?,
        *buf.get(5)?,
        *buf.get(6)?,
        *buf.get(7)?,
    ]);
    *buf = buf.get(8..)?;
    Some(n)
}
fn read_element(buf: &mut &[u8]) -> Option<(TrayElement, bool)> {
    let mut elementbuf = [0; _];
    elementbuf.copy_from_slice(buf.get(..TrayElement::SERIALIZED_BYTESIZE)?);
    let (text, root) = TrayElement::deserialize(elementbuf)?;
    *buf = buf.get(TrayElement::SERIALIZED_BYTESIZE..)?;
    Some((text, root))
}
fn read_menu(buf: &mut &[u8]) -> Option<TrayMenu> {
    let mut menu = [MaybeRootTrayElement {
        root: false,
        element: TrayElement::None,
    }; TRAY_MENU_ITEMS_COUNT];
    for menu_item in &mut menu {
        let (element, root) = read_element(buf)?;
        *menu_item = MaybeRootTrayElement { root, element };
    }
    Some(TrayMenu(menu))
}
