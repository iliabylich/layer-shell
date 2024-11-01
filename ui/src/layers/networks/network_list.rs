use crate::{globals::load_widget, layers::Networks, utils::LayerWindow};
use gtk4::{
    prelude::{Cast, DisplayExt, WidgetExt},
    CenterBox, Label,
};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    let settings_row = load_widget::<CenterBox>("NetworkSettingsRow");
    let exit_row = load_widget::<CenterBox>("NetworkExitRow");

    set_on_click(settings_row, |_| {
        Networks::toggle();
        publish(Command::SpawnNetworkEditor);
    });

    set_on_click(exit_row, |_| {
        Networks::toggle();
    });

    subscribe(on_event);

    for row in rows() {
        set_on_click(row, move |label| {
            if let Some(ip) = label.tooltip_text().map(|s| s.to_string()) {
                let original_label = label.label().as_str().to_string();
                copy_to_clipboard(&ip);
                label.set_label("Copied!");
                gtk4::glib::spawn_future_local(async move {
                    gtk4::glib::timeout_future_seconds(1).await;
                    label.set_label(&original_label);
                });
            }
        });
    }
}

fn on_event(event: &Event) {
    if let Event::NetworkList(list) = event {
        for (idx, row) in rows().iter().enumerate() {
            if let Some((name, ip)) = list.get(idx) {
                row.set_visible(true);
                let label = row.start_widget().unwrap().dynamic_cast::<Label>().unwrap();
                label.set_label(&format!("{}: {}", name, ip));
                label.set_tooltip_text(Some(ip));
            } else {
                row.set_visible(false);
            }
        }
    }
}

fn rows() -> [&'static CenterBox; 5] {
    [
        load_widget::<CenterBox>("Network1Row"),
        load_widget::<CenterBox>("Network2Row"),
        load_widget::<CenterBox>("Network3Row"),
        load_widget::<CenterBox>("Network4Row"),
        load_widget::<CenterBox>("Network5Row"),
    ]
}

fn set_on_click<F>(row: &'static CenterBox, f: F)
where
    F: Fn(Label) + 'static,
{
    let ctrl = gtk4::GestureClick::new();
    ctrl.connect_pressed(move |_, _, _, _| {
        let label = row.start_widget().unwrap().dynamic_cast::<Label>().unwrap();
        f(label);
    });
    row.add_controller(ctrl);
}

fn copy_to_clipboard(text: &str) {
    let clipboard = gtk4::gdk::Display::default().unwrap().clipboard();
    clipboard.set_text(text);
}
