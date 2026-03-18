use crate::{FFIArray, ffi::ShortString};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum TrayItem {
    Regular {
        id: i32,
        uuid: ShortString,
        label: ShortString,
    },
    Disabled {
        id: i32,
        uuid: ShortString,
        label: ShortString,
    },
    Checkbox {
        id: i32,
        uuid: ShortString,
        label: ShortString,
        checked: bool,
    },
    Radio {
        id: i32,
        uuid: ShortString,
        label: ShortString,
        selected: bool,
    },
    Nested {
        id: i32,
        uuid: ShortString,
        label: ShortString,
        children: FFIArray<TrayItem>,
    },
    Section {
        children: FFIArray<TrayItem>,
    },
}
