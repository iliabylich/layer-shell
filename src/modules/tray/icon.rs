use crate::{CArray, CString};

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
        f.debug_struct("TrayIconPixmap")
            .field("w", &self.width)
            .field("h", &self.height)
            .field("bytes", &format!("[...{} bytes]", self.bytes.len))
            .finish()
    }
}
