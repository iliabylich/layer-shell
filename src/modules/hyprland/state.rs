use crate::{event::Event, modules::hyprland::event::HyprlandEvent};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct HyprlandState {
    workspace_ids: HashSet<u64>,
    active_workspace_id: u64,
    lang: String,
}

impl HyprlandState {
    pub(crate) fn init_workspace_ids(&mut self, workspace_ids: HashSet<u64>) {
        self.workspace_ids = workspace_ids;
    }
    pub(crate) fn init_active_workspace(&mut self, active_workspace_id: u64) {
        self.active_workspace_id = active_workspace_id
    }
    pub(crate) fn init_language(&mut self, lang: String) {
        self.lang = lang
    }

    pub(crate) fn initial_events(&self) -> [Event; 2] {
        [self.workspaces_event(), self.language_event()]
    }

    fn workspaces_event(&self) -> Event {
        let workspaces = (1..=10)
            .map(|id| HyprlandWorkspace {
                visible: id <= 5 || self.workspace_ids.contains(&id),
                active: self.active_workspace_id == id,
            })
            .collect::<Vec<_>>()
            .into();

        Event::Workspaces { workspaces }
    }
    fn language_event(&self) -> Event {
        Event::Language {
            lang: self.lang.clone().into(),
        }
    }

    pub(crate) fn apply(&mut self, event: HyprlandEvent) -> Event {
        match event {
            HyprlandEvent::CreateWorkspace(id) => self.add_workspace(id),
            HyprlandEvent::DestroyWorkspace(id) => self.remove_workspace(id),
            HyprlandEvent::Workspace(id) => self.set_active_workspace(id),
            HyprlandEvent::LanguageChanged(lang) => self.set_language(lang),
        }
    }

    fn add_workspace(&mut self, id: u64) -> Event {
        self.workspace_ids.insert(id);
        self.workspaces_event()
    }
    fn remove_workspace(&mut self, id: u64) -> Event {
        self.workspace_ids.remove(&id);
        self.workspaces_event()
    }
    fn set_active_workspace(&mut self, id: u64) -> Event {
        self.active_workspace_id = id;
        self.workspaces_event()
    }
    fn set_language(&mut self, lang: String) -> Event {
        self.lang = lang;
        self.language_event()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HyprlandWorkspace {
    pub visible: bool,
    pub active: bool,
}
