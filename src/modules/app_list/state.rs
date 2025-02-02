use crate::{event::App, modules::app_list::system_app::SystemApp, Event};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct State {
    selected_idx: usize,
    apps: Vec<SystemApp>,
    pattern: String,
}

impl State {
    const MAX_ITEMS: usize = 5;

    pub(crate) fn new() -> Self {
        Self {
            selected_idx: 0,
            apps: vec![],
            pattern: String::new(),
        }
    }

    pub(crate) fn go_up(&mut self) -> Result<()> {
        if self.selected_idx == 0 {
            return Ok(());
        }
        self.selected_idx = std::cmp::max(0, self.selected_idx - 1);
        self.emit();
        Ok(())
    }

    pub(crate) fn go_down(&mut self) -> Result<()> {
        self.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, self.selected_idx + 1);
        self.emit();
        Ok(())
    }

    pub(crate) fn set_search(&mut self, pattern: String) -> Result<()> {
        self.selected_idx = 0;
        self.pattern = pattern;
        self.emit();
        Ok(())
    }

    pub(crate) fn exec_selected(&self) -> Result<()> {
        if let Some(app) = self.visible_apps().get(self.selected_idx) {
            app.exec()?;
        }
        Ok(())
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        self.pattern = String::new();
        self.selected_idx = 0;
        self.apps = SystemApp::parse_all()?;
        self.emit();
        Ok(())
    }

    fn emit(&self) {
        let apps = self
            .visible_apps()
            .into_iter()
            .enumerate()
            .map(|(idx, app)| App {
                name: app.name.into(),
                selected: idx == self.selected_idx,
                icon: app.icon,
            })
            .collect::<Vec<_>>();

        let event = Event::AppList { apps: apps.into() };
        event.emit();
    }

    fn visible_apps(&self) -> Vec<SystemApp> {
        let pattern = self.pattern.to_lowercase();
        self.apps
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&pattern))
            .take(Self::MAX_ITEMS)
            .cloned()
            .collect()
    }
}
