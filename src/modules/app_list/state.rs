use crate::{event::App, modules::app_list::system_app::SystemApp, Event};
use anyhow::{anyhow, Result};
use std::sync::{LazyLock, Mutex};

pub(crate) struct State {
    selected_idx: usize,
    apps: Vec<SystemApp>,
    pattern: String,
}
static INSTANCE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::new()));

impl State {
    const MAX_ITEMS: usize = 5;

    fn new() -> Self {
        Self {
            selected_idx: 0,
            apps: vec![],
            pattern: String::new(),
        }
    }

    fn with<F>(f: F) -> Result<()>
    where
        F: FnOnce(&mut State) -> Result<()>,
    {
        let mut this = INSTANCE.lock().map_err(|_| anyhow!("lock is poisoned"))?;

        f(&mut this)
    }

    pub(crate) fn go_up() -> Result<()> {
        Self::with(|state| {
            if state.selected_idx == 0 {
                return Ok(());
            }
            state.selected_idx = std::cmp::max(0, state.selected_idx - 1);
            state.emit();
            Ok(())
        })
    }

    pub(crate) fn go_down() -> Result<()> {
        Self::with(|state| {
            state.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, state.selected_idx + 1);
            state.emit();
            Ok(())
        })
    }

    pub(crate) fn set_search(pattern: String) -> Result<()> {
        Self::with(|state| {
            state.selected_idx = 0;
            state.pattern = pattern;
            state.emit();
            Ok(())
        })
    }

    pub(crate) fn exec_selected() -> Result<()> {
        Self::with(|state| {
            if let Some(app) = state.visible_apps().get(state.selected_idx) {
                app.exec()?;
            }
            Ok(())
        })
    }

    pub(crate) fn reset() -> Result<()> {
        Self::with(|state| {
            state.pattern = String::new();
            state.selected_idx = 0;
            state.apps = SystemApp::parse_all()?;
            state.emit();
            Ok(())
        })
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
