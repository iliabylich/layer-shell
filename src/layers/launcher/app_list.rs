use crate::{
    globals::load_widget, layers::Launcher, models::AppList as AppListModel, utils::ToggleWindow,
};
use gtk4::{
    prelude::{Cast, EditableExt, WidgetExt},
    Image, Label, SearchEntry,
};

type Output = (Box<dyn Fn()>, Box<dyn Fn(&str)>);

fn rows() -> [&'static gtk4::Box; 5] {
    [
        load_widget::<gtk4::Box>("LauncherRow1"),
        load_widget::<gtk4::Box>("LauncherRow2"),
        load_widget::<gtk4::Box>("LauncherRow3"),
        load_widget::<gtk4::Box>("LauncherRow4"),
        load_widget::<gtk4::Box>("LauncherRow5"),
    ]
}

pub(crate) fn init() -> Output {
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
            Launcher::toggle();
        }
    });
    entry.connect_changed(|_| {
        AppListModel::set_search(entry.text().as_str());
    });

    (
        Box::new(|| {
            AppListModel::reset();
            entry.set_text("");
        }),
        Box::new(|key| match key {
            "Up" => AppListModel::up(),
            "Down" => AppListModel::down(),
            _ => {}
        }),
    )
}
