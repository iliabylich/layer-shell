use crate::Event;
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
    ) -> (Self, Vec<Event>) {
        let state = Self {
            workspace_ids,
            active_workspace_id,
            lang,
        };

        let events = vec![state.workspaces_event(), state.language_event()];

        (state, events)
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

    pub(crate) fn add_workspace(&mut self, id: usize) -> Event {
        self.workspace_ids.insert(id);
        self.workspaces_event()
    }
    pub(crate) fn remove_workspace(&mut self, id: usize) -> Event {
        self.workspace_ids.remove(&id);
        self.workspaces_event()
    }
    pub(crate) fn set_active_workspace(&mut self, id: usize) -> Event {
        self.active_workspace_id = id;
        self.workspaces_event()
    }
    pub(crate) fn set_language(&mut self, lang: String) -> Event {
        self.lang = lang;
        self.language_event()
    }
}
