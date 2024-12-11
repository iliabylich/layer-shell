use crate::{system_app::SystemApp, App, AppList, Event};
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) struct State {
    selected_idx: usize,
    apps: Vec<SystemApp>,
    pattern: String,
}
static mut BUS: Option<Sender<Event>> = None;
static mut STATE: Option<State> = None;

impl State {
    const MAX_ITEMS: usize = 5;

    pub(crate) fn instance() -> &'static mut State {
        unsafe {
            match STATE.as_mut() {
                Some(state) => state,
                None => {
                    log::error!("STATE is not set");
                    std::process::exit(1);
                }
            }
        }
    }

    fn bus() -> &'static Sender<Event> {
        unsafe {
            match BUS.as_ref() {
                Some(bus) => bus,
                None => {
                    log::error!("BUS is not set");
                    std::process::exit(1);
                }
            }
        }
    }

    pub(crate) fn setup() -> Receiver<Event> {
        let tx;
        let rx;
        unsafe {
            STATE = Some(Self::new());
            (tx, rx) = tokio::sync::mpsc::channel(100);
            BUS = Some(tx);
        }
        rx
    }

    fn new() -> Self {
        Self {
            selected_idx: 0,
            apps: vec![],
            pattern: String::new(),
        }
    }

    pub(crate) async fn go_up(&mut self) {
        if self.selected_idx == 0 {
            return;
        }
        self.selected_idx = std::cmp::max(0, self.selected_idx - 1);
        self.emit().await;
    }
    pub(crate) async fn go_down(&mut self) {
        self.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, self.selected_idx + 1);
        self.emit().await;
    }
    pub(crate) async fn set_search(&mut self, pattern: String) {
        self.selected_idx = 0;
        self.pattern = pattern;
        self.emit().await;
    }
    pub(crate) async fn exec_selected(&mut self) {
        if let Some(app) = self.visible_apps().get(self.selected_idx) {
            app.exec();
        }
    }
    pub(crate) async fn reset(&mut self) {
        self.pattern = String::new();
        self.selected_idx = 0;
        match SystemApp::parse_all().await {
            Ok(apps) => self.apps = apps,
            Err(err) => log::error!("failed to refresh app list: {:?}", err),
        }
        self.emit().await;
    }

    async fn emit(&self) {
        let apps = self
            .visible_apps()
            .into_iter()
            .enumerate()
            .map(|(idx, app)| App {
                name: app.name,
                selected: idx == self.selected_idx,
                icon: app.icon,
            })
            .collect::<Vec<_>>();

        if Self::bus()
            .send(Event::AppList(AppList { apps }))
            .await
            .is_err()
        {
            log::error!("failed to send State event");
        }
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
