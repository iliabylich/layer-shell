use ffi::{CArray, CString};

#[derive(Debug)]
pub enum HyprlandEvent {
    Workspaces(WorkspacesEvent),
    Language(LanguageEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct WorkspacesEvent {
    pub workspaces: CArray<Workspace>,
}

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub visible: bool,
    pub active: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct LanguageEvent {
    pub lang: CString,
}
