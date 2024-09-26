use std::collections::HashSet;

use crate::utils::{exec_async, singleton, HyprlandClient, HyprlandEvent};

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

singleton!(HyprlandWorkspaces);

impl HyprlandWorkspaces {
    pub(crate) fn subscribe<F>(min_workspaces: usize, on_change: F)
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
                this().workspace_ids.insert(idx);
                this().changed();
            }
            HyprlandEvent::DestroyWorkspace(idx) => {
                this().workspace_ids.remove(&idx);
                this().changed();
            }
            HyprlandEvent::Workspace(idx) => {
                this().active_id = idx;
                this().changed();
            }
            _ => {}
        });

        gtk4::glib::spawn_future_local(async {
            let (workspace_ids, active_id) = Self::load_initial_data().await;
            this().workspace_ids = workspace_ids;
            this().active_id = active_id;
            this().changed();
        });
    }

    pub(crate) fn go_to(idx: usize) {
        gtk4::glib::spawn_future_local(async move {
            exec_async(&["hyprctl", "dispatch", "workspace", &format!("{idx}")]).await;
        });
    }

    async fn load_initial_data() -> (HashSet<usize>, usize) {
        let workspaces = HyprlandClient::get_workspaces().await;
        let active_workspace = HyprlandClient::get_active_workspace().await;
        (
            HashSet::from_iter(workspaces.into_iter().map(|w| w.id)),
            active_workspace.id,
        )
    }

    fn changed(&self) {
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

        (self.on_change)(workspaces)
    }
}
