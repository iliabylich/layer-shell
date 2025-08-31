use ffi::{CArray, CString};

#[derive(Debug)]
#[repr(C)]
pub struct View {
    pub created_at: CString,
    pub tasks: CArray<Task>,
    pub running: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct Task {
    pub title: CString,
    pub uuid: CString,
    pub duration: CString,
    pub created_at: CString,
    pub selected: bool,
}
