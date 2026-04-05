use crate::{FFIArray, utils::StringRef};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum TrayItem {
    Regular {
        id: i32,
        uuid: StringRef,
        label: StringRef,
    },
    Disabled {
        id: i32,
        uuid: StringRef,
        label: StringRef,
    },
    Checkbox {
        id: i32,
        uuid: StringRef,
        label: StringRef,
        checked: bool,
    },
    Radio {
        id: i32,
        uuid: StringRef,
        label: StringRef,
        selected: bool,
    },
    Nested {
        id: i32,
        uuid: StringRef,
        label: StringRef,
        children: FFIArray<TrayItem>,
    },
    Section {
        children: FFIArray<TrayItem>,
    },
}
