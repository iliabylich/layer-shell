use ffi::{CArray, CString};

#[derive(Debug)]
#[repr(C)]
pub enum TrayEvent {
    AppAdded(TrayAppAddedEvent),
    AppRemoved(TrayAppRemovedEvent),
    AppIconUpdated(TrayAppIconUpdatedEvent),
    AppMenuUpdated(TrayAppMenuUpdatedEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppAddedEvent {
    pub service: CString,
    pub items: CArray<TrayItem>,
    pub icon: TrayIcon,
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppRemovedEvent {
    pub service: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppIconUpdatedEvent {
    pub service: CString,
    pub icon: TrayIcon,
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppMenuUpdatedEvent {
    pub service: CString,
    pub items: CArray<TrayItem>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum TrayItem {
    Regular {
        id: i32,
        uuid: CString,
        label: CString,
    },
    Disabled {
        id: i32,
        uuid: CString,
        label: CString,
    },
    Checkbox {
        id: i32,
        uuid: CString,
        label: CString,
        checked: bool,
    },
    Radio {
        id: i32,
        uuid: CString,
        label: CString,
        selected: bool,
    },
    Nested {
        id: i32,
        uuid: CString,
        label: CString,
        children: CArray<TrayItem>,
    },
    Section {
        children: CArray<TrayItem>,
    },
}

#[derive(Clone)]
#[repr(C)]
pub enum TrayIcon {
    Path { path: CString },
    Name { name: CString },
    PixmapVariant { w: u32, h: u32, bytes: CArray<u8> },
    Unset,
}

impl std::fmt::Debug for TrayIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayIcon::Path { path } => f.debug_struct("TrayIconPath").field("path", path).finish(),
            TrayIcon::Name { name } => f.debug_struct("TrayIconName").field("name", name).finish(),
            TrayIcon::PixmapVariant { w, h, bytes } => f
                .debug_struct("TrayIconPixmapVariant")
                .field("w", w)
                .field("h", h)
                .field("bytes", &format!("[...{} bytes]", bytes.len))
                .finish(),
            TrayIcon::Unset => write!(f, "None"),
        }
    }
}

impl From<String> for TrayIcon {
    fn from(name_or_path: String) -> Self {
        if name_or_path.starts_with("/") {
            Self::Path {
                path: name_or_path.into(),
            }
        } else {
            Self::Name {
                name: name_or_path.into(),
            }
        }
    }
}

impl From<(i32, i32, Vec<u8>)> for TrayIcon {
    fn from((w, h, bytes): (i32, i32, Vec<u8>)) -> Self {
        Self::PixmapVariant {
            w: w as u32,
            h: h as u32,
            bytes: bytes.into(),
        }
    }
}
