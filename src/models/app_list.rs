use crate::utils::{exec_async, singleton};
use gtk4::{
    gdk::AppLaunchContext,
    gio::{DesktopAppInfo, Icon},
    prelude::AppInfoExt,
};

const MAX_ITEMS: usize = 5;

pub(crate) struct AppList {
    selected_idx: usize,
    global_app_list: Vec<App>,
    visible_app_list: Vec<App>,
    on_change: Box<dyn Fn(Vec<AppRow>)>,
}
singleton!(AppList);

fn exclude(app: &DesktopAppInfo) -> bool {
    matches!(app.name().as_str(), "Google Chrome")
}

impl AppList {
    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(Vec<AppRow>) + 'static,
    {
        Self::set(Self {
            selected_idx: 0,
            global_app_list: vec![],
            visible_app_list: vec![],
            on_change: Box::new(f),
        });

        Self::get().refresh_global_app_list();
        Self::get().refresh_visible_app_list("");
        Self::changed();
    }

    pub(crate) fn up() {
        if Self::get().selected_idx == 0 {
            return;
        }
        Self::get().selected_idx = std::cmp::max(0, Self::get().selected_idx - 1);
        Self::changed();
    }
    pub(crate) fn down() {
        Self::get().selected_idx = std::cmp::min(MAX_ITEMS - 1, Self::get().selected_idx + 1);
        Self::changed();
    }
    pub(crate) fn set_search(q: &str) {
        Self::get().selected_idx = 0;
        Self::get().refresh_visible_app_list(q);
        Self::changed();
    }
    pub(crate) fn run_selected() -> bool {
        if let Some(app) = Self::get().visible_app_list.get(Self::get().selected_idx) {
            app.launch();
            true
        } else {
            false
        }
    }
    pub(crate) fn reset() {
        Self::get().selected_idx = 0;
        Self::get().refresh_global_app_list();
        Self::get().refresh_visible_app_list("");
    }

    fn refresh_global_app_list(&mut self) {
        let mut apps = vec![App::Custom {
            name: "Google Chrome (wayland, gtk4)",
            icon_name: "google-chrome",
            launch_args: &[
                "google-chrome-stable",
                "--gtk-version=4",
                "--ozone-platform-hint=wayland",
            ],
        }];

        let builtin_apps = gtk4::gio::AppInfo::all()
            .into_iter()
            .filter(|app_info| app_info.should_show())
            .filter_map(|app_info| app_info.id())
            .filter_map(|app_id| gtk4::gio::DesktopAppInfo::new(app_id.as_str()))
            .filter(|app| !exclude(app))
            .map(App::Default);

        apps.extend(builtin_apps);

        self.global_app_list = apps;
    }
    fn refresh_visible_app_list(&mut self, q: &str) {
        let pattern = q.to_lowercase();

        self.visible_app_list = self
            .global_app_list
            .iter()
            .filter(|app| app.name().to_lowercase().contains(&pattern))
            .take(MAX_ITEMS)
            .cloned()
            .collect::<Vec<_>>();
    }
    fn changed() {
        let apps = Self::get()
            .visible_app_list
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, app)| AppRow {
                app,
                selected: idx == Self::get().selected_idx,
            })
            .collect::<Vec<_>>();
        (Self::get().on_change)(apps);
    }
}

#[derive(Debug, Clone)]
pub(crate) enum App {
    Default(DesktopAppInfo),
    Custom {
        name: &'static str,
        icon_name: &'static str,
        launch_args: &'static [&'static str],
    },
}

impl App {
    fn name(&self) -> String {
        match self {
            App::Default(app) => app.name().to_string(),
            App::Custom { name, .. } => name.to_string(),
        }
    }
    fn icon(&self) -> Option<Icon> {
        match self {
            App::Default(app) => app.icon(),
            App::Custom { icon_name, .. } => Icon::for_string(icon_name).ok(),
        }
    }
    fn launch(&self) {
        match self {
            App::Default(app) => {
                app.launch(&[], AppLaunchContext::NONE).unwrap();
            }
            App::Custom { launch_args, .. } => {
                let launch_args = launch_args.to_vec();
                gtk4::glib::spawn_future_local(async move {
                    exec_async(&launch_args).await;
                });
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct AppRow {
    app: App,
    pub(crate) selected: bool,
}

impl AppRow {
    pub(crate) fn icon(&self) -> Option<Icon> {
        self.app.icon()
    }
    pub(crate) fn name(&self) -> String {
        self.app.name()
    }
}
