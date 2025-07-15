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

#[derive(Clone, Debug)]
#[repr(C)]
pub enum TrayIcon {
    Path { path: CString },
    Name { name: CString },
    Pixmap(TrayIconPixmap),
    Unset,
}

impl TrayIcon {
    pub(crate) fn detect_name_or_path(name_or_path: String) -> Self {
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

#[derive(Clone)]
#[repr(C)]
pub struct TrayIconPixmap {
    pub width: i32,
    pub height: i32,
    pub bytes: CArray<u8>,
}

impl std::fmt::Debug for TrayIconPixmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIconPixmapVariant")
            .field("w", &self.width)
            .field("h", &self.height)
            .field("bytes", &format!("[...{} bytes]", self.bytes.len))
            .finish()
    }
}
