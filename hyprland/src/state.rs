use crate::{HyprlandEvent, LanguageEvent, WorkspacesEvent, reader::ReaderEvent};
use std::collections::HashSet;

pub(crate) struct State {
    workspace_ids: HashSet<usize>,
    active_workspace_id: usize,
    lang: String,
}

impl State {
    pub(crate) fn new(
        workspace_ids: HashSet<usize>,
        active_workspace_id: usize,
        lang: String,
    ) -> Self {
        Self {
            workspace_ids,
            active_workspace_id,
            lang,
        }
    }

    pub(crate) fn initial_events(&self) -> [HyprlandEvent; 2] {
        [self.workspaces_event(), self.language_event()]
    }

    fn workspaces_event(&self) -> HyprlandEvent {
        HyprlandEvent::Workspaces(WorkspacesEvent {
            workspaces: self
                .workspace_ids
                .iter()
                .copied()
                .collect::<Vec<_>>()
                .into(),
            active_workspace: self.active_workspace_id,
        })
    }
    fn language_event(&self) -> HyprlandEvent {
        HyprlandEvent::Language(LanguageEvent {
            lang: self.lang.clone().into(),
        })
    }

    pub(crate) fn apply(&mut self, event: ReaderEvent) -> HyprlandEvent {
        match event {
            ReaderEvent::CreateWorkspace(id) => self.add_workspace(id),
            ReaderEvent::DestroyWorkspace(id) => self.remove_workspace(id),
            ReaderEvent::Workspace(id) => self.set_active_workspace(id),
            ReaderEvent::LanguageChanged(lang) => self.set_language(lang),
        }
    }

    fn add_workspace(&mut self, id: usize) -> HyprlandEvent {
        self.workspace_ids.insert(id);
        self.workspaces_event()
    }
    fn remove_workspace(&mut self, id: usize) -> HyprlandEvent {
        self.workspace_ids.remove(&id);
        self.workspaces_event()
    }
    fn set_active_workspace(&mut self, id: usize) -> HyprlandEvent {
        self.active_workspace_id = id;
        self.workspaces_event()
    }
    fn set_language(&mut self, lang: String) -> HyprlandEvent {
        self.lang = lang;
        self.language_event()
    }
}
