use crate::{Event, reader::ReaderEvent};
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

    pub(crate) fn initial_events(&self) -> [Event; 2] {
        [self.workspaces_event(), self.language_event()]
    }

    fn workspaces_event(&self) -> Event {
        Event::Workspaces {
            ids: self.workspace_ids.iter().copied().collect(),
            active_id: self.active_workspace_id,
        }
    }
    fn language_event(&self) -> Event {
        Event::Language {
            lang: self.lang.clone(),
        }
    }

    pub(crate) fn apply(&mut self, event: ReaderEvent) -> Event {
        match event {
            ReaderEvent::CreateWorkspace(id) => self.add_workspace(id),
            ReaderEvent::DestroyWorkspace(id) => self.remove_workspace(id),
            ReaderEvent::Workspace(id) => self.set_active_workspace(id),
            ReaderEvent::LanguageChanged(lang) => self.set_language(lang),
        }
    }

    fn add_workspace(&mut self, id: usize) -> Event {
        self.workspace_ids.insert(id);
        self.workspaces_event()
    }
    fn remove_workspace(&mut self, id: usize) -> Event {
        self.workspace_ids.remove(&id);
        self.workspaces_event()
    }
    fn set_active_workspace(&mut self, id: usize) -> Event {
        self.active_workspace_id = id;
        self.workspaces_event()
    }
    fn set_language(&mut self, lang: String) -> Event {
        self.lang = lang;
        self.language_event()
    }
}
