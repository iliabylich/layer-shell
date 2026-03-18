use crate::{FFIArray, ffi::ShortString};

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub enum TrayIcon {
    Path {
        path: ShortString,
    },
    Name {
        name: ShortString,
    },
    Pixmap(TrayIconPixmap),
    #[default]
    Unset,
}

impl TrayIcon {
    pub(crate) fn detect_name_or_path(name_or_path: &str) -> Self {
        if name_or_path.starts_with("/") {
            Self::Path {
                path: ShortString::from(name_or_path),
            }
        } else {
            Self::Name {
                name: ShortString::from(name_or_path),
            }
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct TrayIconPixmap {
    pub width: i32,
    pub height: i32,
    pub bytes: FFIArray<u8>,
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
