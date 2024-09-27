use crate::{globals::load_widget, layers::Launcher, models::AppList, utils::LayerWindow};
use gtk4::{
    prelude::{EditableExt, WidgetExt},
    Image, Label, SearchEntry,
};

type Output = (Box<dyn Fn()>, Box<dyn Fn(&str)>);

pub(crate) fn init() -> Output {
    let entry = load_widget::<SearchEntry>("LauncherEntry");
    let rows = [
        load_widget::<gtk4::Box>("LauncherRow1"),
        load_widget::<gtk4::Box>("LauncherRow2"),
        load_widget::<gtk4::Box>("LauncherRow3"),
        load_widget::<gtk4::Box>("LauncherRow4"),
        load_widget::<gtk4::Box>("LauncherRow5"),
    ];
    let images = [
        load_widget::<Image>("LauncherRow1Image"),
        load_widget::<Image>("LauncherRow2Image"),
        load_widget::<Image>("LauncherRow3Image"),
        load_widget::<Image>("LauncherRow4Image"),
        load_widget::<Image>("LauncherRow5Image"),
    ];
    let labels = [
        load_widget::<Label>("LauncherRow1Label"),
        load_widget::<Label>("LauncherRow2Label"),
        load_widget::<Label>("LauncherRow3Label"),
        load_widget::<Label>("LauncherRow4Label"),
        load_widget::<Label>("LauncherRow5Label"),
    ];

    AppList::subscribe(move |apps| {
        for idx in 0..5 {
            let row = rows[idx];
            let image = images[idx];
            let label = labels[idx];
            if let Some(app) = apps.get(idx) {
                row.set_visible(true);
                if app.selected {
                    row.add_css_class("active");
                } else {
                    row.remove_css_class("active");
                }

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
        if AppList::run_selected() {
            Launcher::toggle();
        }
    });
    entry.connect_changed(|_| {
        AppList::set_search(entry.text().as_str());
    });

    (
        Box::new(|| {
            AppList::reset();
            entry.set_text("");
        }),
        Box::new(|key| match key {
            "Up" => AppList::up(),
            "Down" => AppList::down(),
            _ => {}
        }),
    )
}
