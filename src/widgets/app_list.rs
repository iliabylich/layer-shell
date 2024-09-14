use gtk4::{
    prelude::{Cast, EditableExt, WidgetExt},
    Image, Label, SearchEntry,
};

use crate::{
    globals::{load_widget, toggle_window},
    models::AppList as AppListModel,
};

pub(crate) struct AppList {
    pub(crate) reset: Box<dyn Fn()>,
    pub(crate) on_key_press: Box<dyn Fn(&str)>,
}

fn rows() -> [&'static gtk4::Box; 5] {
    [
        load_widget::<gtk4::Box>("LauncherRow1"),
        load_widget::<gtk4::Box>("LauncherRow2"),
        load_widget::<gtk4::Box>("LauncherRow3"),
        load_widget::<gtk4::Box>("LauncherRow4"),
        load_widget::<gtk4::Box>("LauncherRow5"),
    ]
}

impl AppList {
    pub(crate) fn init() -> Self {
        let entry = load_widget::<SearchEntry>("LauncherEntry");

        AppListModel::spawn(5, |apps| {
            for (idx, row) in rows().iter().enumerate() {
                if let Some(app) = apps.get(idx) {
                    row.set_visible(true);
                    if app.selected {
                        row.add_css_class("active");
                    } else {
                        row.remove_css_class("active");
                    }
                    let image = row.first_child().unwrap().dynamic_cast::<Image>().unwrap();
                    let label = row.last_child().unwrap().dynamic_cast::<Label>().unwrap();

                    if let Some(icon) = app.icon() {
                        image.set_from_gicon(&icon);
                    }
                    label.set_label(&app.name());
                } else {
                    row.set_visible(false);
                }
            }
        });

        entry.connect_activate(|_| {
            if AppListModel::run_selected() {
                toggle_window("Launcher");
            }
        });
        entry.connect_changed(|_| {
            AppListModel::set_search(entry.text().as_str());
        });

        Self {
            reset: Box::new(|| {
                AppListModel::reset();
                entry.set_text("");
            }),
            on_key_press: Box::new(|key| match key {
                "Up" => AppListModel::up(),
                "Down" => AppListModel::down(),
                _ => {}
            }),
        }
    }
}
