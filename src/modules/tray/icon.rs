use crate::{FFIArray, utils::StringRef};

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub enum TrayIcon {
    Path {
        path: StringRef,
    },
    Name {
        name: StringRef,
    },
    Pixmap(TrayIconPixmap),
    #[default]
    Unset,
}

impl TrayIcon {
    pub(crate) fn detect_name_or_path(name_or_path: &str) -> Self {
        if name_or_path.starts_with("/") {
            Self::Path {
                path: StringRef::new(name_or_path),
            }
        } else {
            Self::Name {
                name: StringRef::new(name_or_path),
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

impl core::fmt::Debug for TrayIconPixmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrayIconPixmap")
            .field("w", &self.width)
            .field("h", &self.height)
            .field("bytes", &format!("[...{} bytes]", self.bytes.len))
            .finish()
    }
}
