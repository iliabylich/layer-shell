use ffi::{CArray, CString};

#[derive(Debug)]
pub enum HyprlandEvent {
    Workspaces(WorkspacesEvent),
    Language(LanguageEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct WorkspacesEvent {
    pub ids: CArray<usize>,
    pub active_id: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct LanguageEvent {
    pub lang: CString,
}
