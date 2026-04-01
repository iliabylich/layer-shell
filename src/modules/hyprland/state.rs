use crate::{event::Event, ffi::ShortString};
use core::cell::RefCell;
use std::{collections::HashSet, rc::Rc};

pub(crate) struct HyprlandState {
    inner: Rc<RefCell<Inner>>,
}

pub(crate) enum HyprlandDiff {
    SetWorkspaceIds(HashSet<u64>),
    AddWorkspaceId(u64),
    RemoveWorkspaceId(u64),

    SetActiveWorkspaceId(u64),
    SetLanguage(ShortString),

    SetCapsLockEnabled(bool),
}

impl HyprlandState {
    pub(crate) fn empty() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::empty())),
        }
    }

    pub(crate) fn apply(&self, diff: HyprlandDiff) -> Option<Event> {
        let mut inner = self.inner.borrow_mut();
        inner.apply(diff)
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

struct Inner {
    workspace_ids: Option<HashSet<u64>>,
    active_workspace_id: Option<u64>,
    lang: Option<ShortString>,
}

impl Inner {
    fn empty() -> Self {
        Self {
            workspace_ids: None,
            active_workspace_id: None,
            lang: None,
        }
    }

    fn apply(&mut self, diff: HyprlandDiff) -> Option<Event> {
        enum Changed {
            Workspaces,
            Language,
        }
        let changed;

        match diff {
            HyprlandDiff::SetWorkspaceIds(workspace_ids) => {
                self.workspace_ids = Some(workspace_ids);
                changed = Changed::Workspaces;
            }
            HyprlandDiff::AddWorkspaceId(workspace_id) => {
                if let Some(workspace_ids) = &mut self.workspace_ids {
                    workspace_ids.insert(workspace_id);
                }
                changed = Changed::Workspaces;
            }
            HyprlandDiff::RemoveWorkspaceId(workspace_id) => {
                if let Some(workspace_ids) = &mut self.workspace_ids {
                    workspace_ids.remove(&workspace_id);
                }
                changed = Changed::Workspaces;
            }
            HyprlandDiff::SetActiveWorkspaceId(active_workspace_id) => {
                self.active_workspace_id = Some(active_workspace_id);
                changed = Changed::Workspaces;
            }
            HyprlandDiff::SetLanguage(lang) => {
                self.lang = Some(lang);
                changed = Changed::Language;
            }
            HyprlandDiff::SetCapsLockEnabled(enabled) => {
                return Some(Event::CapsLockToggled { enabled });
            }
        }

        match changed {
            Changed::Workspaces => {
                let (active_workspace_id, workspace_ids) =
                    self.active_workspace_id.zip(self.workspace_ids.as_ref())?;
                let workspaces = (1..=10)
                    .map(|id| HyprlandWorkspace {
                        visible: id <= 5 || workspace_ids.contains(&id),
                        active: active_workspace_id == id,
                    })
                    .collect::<Vec<_>>()
                    .into();
                Some(Event::Workspaces { workspaces })
            }

            Changed::Language => {
                let lang = self.lang?;
                Some(Event::Language { lang })
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HyprlandWorkspace {
    pub visible: bool,
    pub active: bool,
}
