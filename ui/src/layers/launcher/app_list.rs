use crate::{
    layers::Launcher,
    widgets::{
        LauncherEntry, LauncherRow1, LauncherRow1Image, LauncherRow1Label, LauncherRow2,
        LauncherRow2Image, LauncherRow2Label, LauncherRow3, LauncherRow3Image, LauncherRow3Label,
        LauncherRow4, LauncherRow4Image, LauncherRow4Label, LauncherRow5, LauncherRow5Image,
        LauncherRow5Label,
    },
};
use gtk4::{
    prelude::{EditableExt, WidgetExt},
    Image, Label,
};
use layer_shell_io::{publish, subscribe, AppIcon, Command, Event};

type Output = (Box<dyn Fn()>, Box<dyn Fn(&str)>);

pub(crate) fn init() -> Output {
    let entry = LauncherEntry();
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
        LauncherRow1(),
        LauncherRow2(),
        LauncherRow3(),
        LauncherRow4(),
        LauncherRow5(),
    ]
}

fn images() -> [&'static Image; 5] {
    [
        LauncherRow1Image(),
        LauncherRow2Image(),
        LauncherRow3Image(),
        LauncherRow4Image(),
        LauncherRow5Image(),
    ]
}

fn labels() -> [&'static Label; 5] {
    [
        LauncherRow1Label(),
        LauncherRow2Label(),
        LauncherRow3Label(),
        LauncherRow4Label(),
        LauncherRow5Label(),
    ]
}
