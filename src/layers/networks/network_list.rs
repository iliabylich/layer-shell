use gtk4::{
    prelude::{Cast, DisplayExt, WidgetExt},
    CenterBox, Label,
};

use crate::{
    globals::load_widget,
    layers::Networks,
    models::NetworkList,
    utils::{exec_async, ToggleWindow},
};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    let settings_row = load_widget::<CenterBox>("NetworkSettingsRow");
    let exit_row = load_widget::<CenterBox>("NetworkExitRow");

    set_on_click(settings_row, |_| {
        gtk4::glib::spawn_future_local(async move {
            Networks::toggle();
            exec_async(&["kitty", "--name", "nmtui", "nmtui"]).await;
        });
    });

    set_on_click(exit_row, |_| {
        Networks::toggle();
    });

    for (idx, row) in rows().iter().enumerate() {
        set_on_click(row, move |label| {
            if let Some(network) = NetworkList::get_current().get(idx) {
                copy_to_clipboard(&network.ip);
                label.set_label("Copied!");
                gtk4::glib::spawn_future_local(async move {
                    gtk4::glib::timeout_future_seconds(1).await;
                    label.set_label(&format!("{}: {}", network.name, network.ip));
                });
            }
        });
    }

    (Box::new(sync_ui), Box::new(|_key| {}))
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

fn sync_ui() {
    for (idx, row) in rows().iter().enumerate() {
        if let Some(network) = NetworkList::get_current().get(idx) {
            row.set_visible(true);
            row.start_widget()
                .unwrap()
                .dynamic_cast::<Label>()
                .unwrap()
                .set_label(&format!("{}: {}", network.name, network.ip));
        } else {
            row.set_visible(false);
        }
    }
}
