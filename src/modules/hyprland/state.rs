use crate::event::Event;
use std::collections::HashSet;

pub(crate) struct HyprlandState {
    workspace_ids: Option<HashSet<u64>>,
    active_workspace_id: Option<u64>,
    lang: Option<String>,
}

pub(crate) enum HyprlandDiff {
    SetWorkspaceIds(HashSet<u64>),
    AddWorkspaceId(u64),
    RemoveWorkspaceId(u64),

    SetActiveWorkspaceId(u64),
    SetLanguage(String),

    SetCapsLockEnabled(bool),
}

impl HyprlandState {
    pub(crate) fn empty() -> Self {
        Self {
            workspace_ids: None,
            active_workspace_id: None,
            lang: None,
        }
    }

    pub(crate) fn apply(&mut self, diff: HyprlandDiff) -> Option<Event> {
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
                let lang = self.lang.clone()?;
                Some(Event::Language { lang: lang.into() })
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
