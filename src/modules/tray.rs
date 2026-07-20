use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{ArrayWriter, FixedSizeBuffer, StringRef, StringRefExt, getenv, new_sockaddr_un},
};
use anyhow::{Context, Result};
use core::fmt::Write;
use libc::sockaddr_un;

#[derive(Clone, Copy)]
pub(crate) struct Tray {
    reader: UnixSocketReader,
    writebuf: [u8; 8],
    emitter: Emitter,
}

impl Tray {
    pub(crate) const BUFFER_SIZE: usize = TrayEvent::SERIALIZED_BYTESIZE;

    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let mut buf = [0; 200];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "{xdg_runtime_dir}/tray-mon.sock")?;
        let path = writer.as_bytes()?;
        let addr = new_sockaddr_un(path)?;
        Ok(addr)
    }

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            reader: UnixSocketReader::new(),
            writebuf: [0; _],
            emitter,
        }
    }

    pub(crate) fn wants(
        &mut self,
        addr: &sockaddr_un,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Option<Wants> {
        self.reader.wants(addr, buf.remainder())
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Result<()> {
        if matches!(satisfy, Satisfy::Write { .. }) {
            return Ok(());
        }

        if let Some(written) = self.reader.satisfy(satisfy)?
            && let Some(buf) = buf.written(written)
        {
            let event = TrayEvent::deserialize(&buf).context("failed to deserialize TrayEvent")?;

            let event = match event {
                TrayEvent::AppAdded {
                    service,
                    icon,
                    menu,
                } => IoEvent::TrayAppAdded {
                    service,
                    menu,
                    icon: StringRef::new(icon.as_str()?),
                },
                TrayEvent::AppRemoved { service } => IoEvent::TrayAppRemoved { service },
                TrayEvent::MenuUpdated { service, menu } => {
                    IoEvent::TrayAppMenuUpdated { service, menu }
                }
                TrayEvent::IconUpdated { service, icon } => IoEvent::TrayAppIconUpdated {
                    service,
                    icon: StringRef::new(icon.as_str()?),
                },
            };

            self.emitter.emit(&event);
        }

        Ok(())
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

#[derive(Debug, Clone, Copy)]
#[must_use]
#[expect(clippy::large_enum_variant)]
enum TrayEvent {
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
    const SERIALIZED_BYTESIZE: usize = 1
        + size_of::<u32>()
        + TrayFixedSizeString::SERIALIZED_BYTESIZE
        + TrayMenu::SERIALIZED_BYTESIZE;

    fn deserialize(buf: &[u8; Self::SERIALIZED_BYTESIZE]) -> Option<Self> {
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TrayMenu(pub FixedSizeArrray<TRAY_MENU_ITEMS_COUNT, MaybeRootTrayElement>);

impl TrayMenu {
    const SERIALIZED_BYTESIZE: usize = TRAY_MENU_ITEMS_COUNT * TrayElement::SERIALIZED_BYTESIZE;
}

#[derive(Clone, Copy, Default)]
#[must_use]
#[repr(C)]
pub struct MaybeRootTrayElement {
    root: bool,
    element: TrayElement,
}
impl core::fmt::Debug for MaybeRootTrayElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.root {
            write!(f, "Root({:?})", self.element)
        } else {
            write!(f, "Child({:?})", self.element)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
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
    #[default]
    None,
}
impl TrayElement {
    const SERIALIZED_BYTESIZE: usize = size_of::<u8>()
        + size_of::<i32>()
        + size_of::<u8>()
        + TrayLabel::SERIALIZED_BYTESIZE
        + size_of::<u64>() * 2;

    fn deserialize(buf: [u8; Self::SERIALIZED_BYTESIZE]) -> Option<(Self, bool)> {
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

#[derive(Clone, Copy)]
#[must_use]
#[repr(C)]
pub struct TrayLabel {
    buf: [u8; TRAY_LABEL_BYTESIZE],
    len: u32,
}
impl TrayLabel {
    const SERIALIZED_BYTESIZE: usize = TRAY_LABEL_BYTESIZE + 4;

    fn deserialize(buf: [u8; Self::SERIALIZED_BYTESIZE]) -> Self {
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let mut s = [0; _];
        s.copy_from_slice(&buf[4..]);
        Self { len, buf: s }
    }

    fn as_bytes(&self) -> Result<&[u8]> {
        self.buf
            .get(..self.len as usize)
            .context("malformed TrayFixedSizeString")
    }

    fn as_str(&self) -> Result<&str> {
        let s = core::str::from_utf8(self.as_bytes()?)?;
        Ok(s)
    }
}
impl core::fmt::Debug for TrayLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

#[derive(Clone, Copy)]
#[must_use]
pub(crate) struct TrayFixedSizeString {
    buf: [u8; Self::STR_BYTESIZE],
    len: u32,
}
impl TrayFixedSizeString {
    const STR_BYTESIZE: usize = 256;
    const SERIALIZED_BYTESIZE: usize = Self::STR_BYTESIZE + 4;

    fn deserialize(buf: &[u8; Self::SERIALIZED_BYTESIZE]) -> Self {
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let mut s = [0; _];
        s.copy_from_slice(&buf[4..]);
        Self { len, buf: s }
    }

    fn as_bytes(&self) -> Result<&[u8]> {
        self.buf
            .get(..self.len as usize)
            .context("malformed TrayFixedSizeString")
    }

    fn as_str(&self) -> Result<&str> {
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
    let mut menu = FixedSizeArrray::new();
    for _ in 0..TRAY_MENU_ITEMS_COUNT {
        let (element, root) = read_element(buf)?;
        if matches!(element, TrayElement::None) {
            break;
        }
        menu.push(MaybeRootTrayElement { root, element })
            .unwrap_or_else(|_| unreachable!("constants don't match"));
    }
    Some(TrayMenu(menu))
}
