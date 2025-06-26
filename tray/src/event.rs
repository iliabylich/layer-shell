use ffi::{CArray, CString};

#[derive(Debug)]
#[repr(C)]
pub enum TrayEvent {
    AppUpdated(TrayAppUpdatedEvent),
    AppRemoved(TrayAppRemovedEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppUpdatedEvent {
    pub service: CString,
    pub root_item: TrayItem,
    pub icon: TrayIcon,
}

#[derive(Debug)]
#[repr(C)]
pub struct TrayAppRemovedEvent {
    pub service: CString,
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct TrayItem {
    pub id: i32,
    pub uuid: CString,
    pub type_: CString,
    pub label: CString,
    pub enabled: bool,
    pub visible: bool,
    pub icon_name: CString,
    pub icon_data: CString,
    pub toggle_type: CString,
    pub toggle_state: i32,
    pub children_display: CString,
    pub children: CArray<TrayItem>,
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
