use crate::{CArray, CString};

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
