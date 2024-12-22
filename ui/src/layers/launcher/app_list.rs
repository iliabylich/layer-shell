use crate::{
    layers::Launcher,
    widgets::launcher::{Input, Rows},
};
use gtk4::prelude::{EditableExt, WidgetExt};
use layer_shell_io::{
    app_list::{AppIcon, AppListExecSelected, AppListSetSearch},
    publish, subscribe, Command, Event,
};

pub(crate) fn init() {
    Input().connect_activate(|_| {
        publish(Command::AppListExecSelected(AppListExecSelected));
        Launcher::toggle();
    });
    Input().connect_changed(|entry| {
        let search = entry.text().to_string();
        publish(Command::AppListSetSearch(AppListSetSearch { search }));
    });

    subscribe(|event| {
        if let Event::AppList(event) = event {
            for (idx, (row, image, label)) in Rows().iter().enumerate() {
                if let Some(app) = event.apps.get(idx) {
                    row.set_visible(true);
                    if app.selected {
                        row.add_css_class("active");
                    } else {
                        row.remove_css_class("active");
                    }

                    match &app.icon {
                        AppIcon::IconName(icon) => image.set_icon_name(Some(icon)),
                        AppIcon::IconPath(path) => image.set_from_file(Some(path)),
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
    Input().set_text("");
}
