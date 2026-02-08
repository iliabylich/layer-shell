use crate::{FFIArray, FFIString};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum TrayItem {
    Regular {
        id: i32,
        uuid: FFIString,
        label: FFIString,
    },
    Disabled {
        id: i32,
        uuid: FFIString,
        label: FFIString,
    },
    Checkbox {
        id: i32,
        uuid: FFIString,
        label: FFIString,
        checked: bool,
    },
    Radio {
        id: i32,
        uuid: FFIString,
        label: FFIString,
        selected: bool,
    },
    Nested {
        id: i32,
        uuid: FFIString,
        label: FFIString,
        children: FFIArray<TrayItem>,
    },
    Section {
        children: FFIArray<TrayItem>,
    },
}
