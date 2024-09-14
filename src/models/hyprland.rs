use std::collections::HashSet;

use crate::utils::{exec_async, HyprlandClient, HyprlandEvent};

use super::singleton;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Workspace {
    pub(crate) visible: bool,
    pub(crate) active: bool,
}

pub(crate) struct HyprlandWorkspaces {
    min_workspaces: usize,
    workspace_ids: HashSet<usize>,
    active_id: usize,
    on_change: Box<dyn Fn([Workspace; 10])>,
}

singleton!(HyprlandWorkspaces, WORKSPACES_INSTANCE);

impl HyprlandWorkspaces {
    pub(crate) fn spawn<F>(min_workspaces: usize, on_change: F)
    where
        F: Fn([Workspace; 10]) + 'static,
    {
        Self::set(Self {
            min_workspaces,
            workspace_ids: HashSet::new(),
            active_id: 0,
            on_change: Box::new(on_change),
        });

        HyprlandClient::subscribe(move |event| match event {
            HyprlandEvent::CreateWorkspace(idx) => {
                Self::get().workspace_ids.insert(idx);
                Self::get().changed();
            }
            HyprlandEvent::DestroyWorkspace(idx) => {
                Self::get().workspace_ids.remove(&idx);
                Self::get().changed();
            }
            HyprlandEvent::Workspace(idx) => {
                Self::get().active_id = idx;
                Self::get().changed();
            }
            _ => {}
        });

        gtk4::glib::spawn_future_local(async {
            Self::get().load_initial_data().await;
        });
    }

    #[allow(unused_must_use)]
    pub(crate) fn go_to(idx: usize) {
        gtk4::glib::spawn_future_local(async move {
            exec_async(&["hyprctl", "dispatch", "workspace", &format!("{idx}")]).await;
        });
    }

    async fn resync(&mut self) {
        let workspaces = HyprlandClient::get_workspaces().await;
        self.workspace_ids = HashSet::from_iter(workspaces.into_iter().map(|w| w.id));

        let active_workspace = HyprlandClient::get_active_workspace().await;
        self.active_id = active_workspace.id;
    }
    async fn load_initial_data(&mut self) {
        self.resync().await;
        self.changed();
    }

    fn changed(&self) {
        (self.on_change)(self.get_data())
    }

    fn get_data(&self) -> [Workspace; 10] {
        let mut ids_to_show = HashSet::new();
        for id in self.workspace_ids.iter() {
            ids_to_show.insert(*id);
        }
        ids_to_show.insert(self.active_id);

        // create min required number of workspaces
        for id in 1..=self.min_workspaces {
            ids_to_show.insert(id);
        }

        let mut workspaces = [Workspace::default(); 10];
        for id in 1..=10 {
            workspaces[id - 1] = Workspace {
                visible: ids_to_show.contains(&id),
                active: id == self.active_id,
            }
        }
        workspaces
    }
}

pub(crate) struct HyprlandLanguage {
    on_change: Box<dyn Fn(String)>,
}
singleton!(HyprlandLanguage, LANGUAGE_INSTANCE);

impl HyprlandLanguage {
    pub(crate) fn spawn<F>(f: F)
    where
        F: Fn(String) + 'static,
    {
        Self::set(Self {
            on_change: Box::new(f),
        });

        HyprlandClient::subscribe(|event| {
            if let HyprlandEvent::LanguageChanged(new_lang) = event {
                Self::get().changed(new_lang);
            }
        });

        gtk4::glib::spawn_future_local(async {
            Self::get().load_initial_data().await;
        });
    }

    fn changed(&self, lang: String) {
        (self.on_change)(lang)
    }

    async fn load_initial_data(&self) {
        let devices = HyprlandClient::get_devices().await;
        let layout = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .unwrap()
            .active_keymap;
        self.changed(layout);
    }
}
