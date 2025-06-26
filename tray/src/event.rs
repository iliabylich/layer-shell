use ffi::{CArray, CString};

#[derive(Debug)]
#[repr(C)]
pub struct TrayEvent {
    pub apps: CArray<TrayApp>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TrayApp {
    pub root_item: TrayItem,
    pub icon: TrayIcon,
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
