use crate::{event::App, global::global, modules::app_list::system_app::SystemApp, Event};

pub(crate) struct State {
    selected_idx: usize,
    apps: Vec<SystemApp>,
    pattern: String,
}
global!(INSTANCE, State);

impl State {
    const MAX_ITEMS: usize = 5;

    pub(crate) fn setup() {
        INSTANCE::set(Self::new());
    }

    pub(crate) fn instance() -> &'static mut State {
        INSTANCE::get()
    }

    fn new() -> Self {
        Self {
            selected_idx: 0,
            apps: vec![],
            pattern: String::new(),
        }
    }

    pub(crate) fn go_up(&mut self) {
        if self.selected_idx == 0 {
            return;
        }
        self.selected_idx = std::cmp::max(0, self.selected_idx - 1);
        self.emit();
    }
    pub(crate) fn go_down(&mut self) {
        self.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, self.selected_idx + 1);
        self.emit();
    }
    pub(crate) fn set_search(&mut self, pattern: String) {
        self.selected_idx = 0;
        self.pattern = pattern;
        self.emit();
    }
    pub(crate) fn exec_selected(&mut self) {
        if let Some(app) = self.visible_apps().get(self.selected_idx) {
            app.exec();
        }
    }
    pub(crate) fn reset(&mut self) {
        self.pattern = String::new();
        self.selected_idx = 0;
        match SystemApp::parse_all() {
            Ok(apps) => self.apps = apps,
            Err(err) => log::error!("failed to refresh app list: {:?}", err),
        }
        self.emit();
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
