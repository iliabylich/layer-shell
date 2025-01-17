use std::sync::{LazyLock, Mutex};

use crate::{event::App, modules::app_list::system_app::SystemApp, Event};

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

    pub(crate) fn go_up() {
        let mut this = INSTANCE.lock().unwrap();
        if this.selected_idx == 0 {
            return;
        }
        this.selected_idx = std::cmp::max(0, this.selected_idx - 1);
        this.emit();
    }

    pub(crate) fn go_down() {
        let mut this = INSTANCE.lock().unwrap();
        this.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, this.selected_idx + 1);
        this.emit();
    }

    pub(crate) fn set_search(pattern: String) {
        let mut this = INSTANCE.lock().unwrap();
        this.selected_idx = 0;
        this.pattern = pattern;
        this.emit();
    }

    pub(crate) fn exec_selected() {
        let this = INSTANCE.lock().unwrap();
        if let Some(app) = this.visible_apps().get(this.selected_idx) {
            app.exec();
        }
    }

    pub(crate) fn reset() {
        let mut this = INSTANCE.lock().unwrap();
        this.pattern = String::new();
        this.selected_idx = 0;
        match SystemApp::parse_all() {
            Ok(apps) => this.apps = apps,
            Err(err) => log::error!("failed to refresh app list: {:?}", err),
        }
        this.emit();
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
