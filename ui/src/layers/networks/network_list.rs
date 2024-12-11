use crate::{
    layers::Networks,
    widgets::networks::{ExitRow, Rows, SettingsRow},
};
use gtk4::{
    prelude::{DisplayExt, WidgetExt},
    CenterBox,
};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() {
    set_on_click(SettingsRow(), || {
        Networks::toggle();
        publish(Command::SpawnNetworkEditor);
    });

    set_on_click(ExitRow(), || {
        Networks::toggle();
    });

    subscribe(|event| {
        if let Event::NetworkList(event) = event {
            for (idx, (row, label)) in Rows().iter().enumerate() {
                if let Some(network) = event.list.get(idx) {
                    row.set_visible(true);
                    label.set_label(&format!("{}: {}", network.iface, network.address));
                    label.set_tooltip_text(Some(&network.address));
                } else {
                    row.set_visible(false);
                }
            }
        }
    });

    for (row, label) in Rows() {
        set_on_click(row, move || {
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
    F: Fn() + 'static,
{
    let ctrl = gtk4::GestureClick::new();
    ctrl.connect_pressed(move |_, _, _, _| {
        f();
    });
    row.add_controller(ctrl);
}

fn copy_to_clipboard(text: &str) {
    if let Some(display) = gtk4::gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(text);
    } else {
        log::error!("failed to get default Gdk display");
    }
}
