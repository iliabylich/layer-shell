use crate::{
    globals::load_widget,
    layers::Launcher,
    models::{publish, subscribe, AppIcon, Command, Event},
    utils::LayerWindow,
};
use gtk4::{
    prelude::{EditableExt, WidgetExt},
    Image, Label, SearchEntry,
};

type Output = (Box<dyn Fn()>, Box<dyn Fn(&str)>);

pub(crate) fn init() -> Output {
    let entry = load_widget::<SearchEntry>("LauncherEntry");
    entry.connect_activate(|_| {
        publish(Command::LauncherExecSelected);
        Launcher::toggle();
    });
    entry.connect_changed(|_| {
        let text = entry.text().to_string();
        publish(Command::LauncherSetSearch(text));
    });

    subscribe(on_event);

    (
        Box::new(|| {
            publish(Command::LauncherReset);
            entry.set_text("");
        }),
        Box::new(|key| match key {
            "Up" => publish(Command::LauncherGoUp),
            "Down" => publish(Command::LauncherGoDown),
            _ => {}
        }),
    )
}

fn on_event(event: &Event) {
    if let Event::AppList(apps) = event {
        let rows = rows();
        let images = images();
        let labels = labels();

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

                match &app.icon {
                    AppIcon::IconName(icon) => image.set_icon_name(Some(icon)),
                    AppIcon::IconPath(path) => {
                        image.set_from_file(Some(path));
                    }
                }
                label.set_label(&app.name);
            } else {
                row.set_visible(false);
            }
        }
    }
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

fn images() -> [&'static Image; 5] {
    [
        load_widget::<Image>("LauncherRow1Image"),
        load_widget::<Image>("LauncherRow2Image"),
        load_widget::<Image>("LauncherRow3Image"),
        load_widget::<Image>("LauncherRow4Image"),
        load_widget::<Image>("LauncherRow5Image"),
    ]
}

fn labels() -> [&'static Label; 5] {
    [
        load_widget::<Label>("LauncherRow1Label"),
        load_widget::<Label>("LauncherRow2Label"),
        load_widget::<Label>("LauncherRow3Label"),
        load_widget::<Label>("LauncherRow4Label"),
        load_widget::<Label>("LauncherRow5Label"),
    ]
}
