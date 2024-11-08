use crate::{
    layers::Launcher,
    widgets::{self, LauncherEntry},
};
use gtk4::prelude::{EditableExt, WidgetExt};
use layer_shell_io::{publish, subscribe, AppIcon, Command, Event};

pub(crate) fn init() {
    let entry = LauncherEntry();
    entry.connect_activate(|_| {
        publish(Command::LauncherExecSelected);
        Launcher::toggle();
    });
    entry.connect_changed(|_| {
        let text = entry.text().to_string();
        publish(Command::LauncherSetSearch(text));
    });

    subscribe(|event| {
        if let Event::AppList(apps) = event {
            let rows = widgets::launcher::rows();
            let images = widgets::launcher::images();
            let labels = widgets::launcher::labels();

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
    });
}

pub(crate) fn reset() {
    LauncherEntry().set_text("");
}
