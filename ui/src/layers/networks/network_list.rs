use crate::{
    layers::Networks,
    widgets::{self, NetworkExitRow, NetworkSettingsRow},
};
use gtk4::{
    prelude::{Cast, DisplayExt, WidgetExt},
    CenterBox, Label,
};
use layer_shell_io::{publish, subscribe, Command, Event, Network};

pub(crate) fn init() {
    set_on_click(NetworkSettingsRow(), |_| {
        Networks::toggle();
        publish(Command::SpawnNetworkEditor);
    });

    set_on_click(NetworkExitRow(), |_| {
        Networks::toggle();
    });

    subscribe(|event| {
        if let Event::NetworkList(list) = event {
            for (idx, row) in widgets::networks::rows().iter().enumerate() {
                if let Some(Network { iface, address }) = list.get(idx) {
                    row.set_visible(true);
                    if let Some(label) = row_label(row) {
                        label.set_label(&format!("{}: {}", iface, address));
                        label.set_tooltip_text(Some(address));
                    } else {
                        eprintln!("failed to get network label");
                    }
                } else {
                    row.set_visible(false);
                }
            }
        }
    });

    for row in widgets::networks::rows() {
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

fn set_on_click<F>(row: &'static CenterBox, f: F)
where
    F: Fn(Label) + 'static,
{
    let ctrl = gtk4::GestureClick::new();
    ctrl.connect_pressed(move |_, _, _, _| {
        if let Some(label) = row_label(row) {
            f(label);
        } else {
            eprintln!("failed to get network label");
        }
    });
    row.add_controller(ctrl);
}

fn copy_to_clipboard(text: &str) {
    if let Some(display) = gtk4::gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(text);
    } else {
        eprintln!("failed to get default Gdk display");
    }
}

fn row_label(row: &CenterBox) -> Option<Label> {
    row.start_widget()?.dynamic_cast::<Label>().ok()
}
