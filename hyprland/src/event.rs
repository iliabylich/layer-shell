use ffi::{CArray, CString};

#[derive(Debug)]
pub enum HyprlandEvent {
    Workspaces(WorkspacesEvent),
    Language(LanguageEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct WorkspacesEvent {
    pub workspaces: CArray<usize>,
    pub active_workspace: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct LanguageEvent {
    pub lang: CString,
}
