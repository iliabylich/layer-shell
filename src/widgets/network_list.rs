use gtk4::{
    prelude::{Cast, DisplayExt, WidgetExt},
    CenterBox, Label,
};

use crate::{
    globals::{load_widget, toggle_window},
    models::{all_networks, singleton, Iface},
    utils::exec_async,
};

pub(crate) struct NetworkList {
    pub(crate) reset: Box<dyn Fn()>,
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

struct State(Vec<Iface>);
singleton!(State);
impl State {
    fn list() -> &'static [Iface] {
        &Self::get().0
    }
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
    if !State::is_set() {
        return;
    }

    for (idx, row) in rows().iter().enumerate() {
        if let Some(network) = State::list().get(idx) {
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

impl NetworkList {
    pub(crate) fn init() -> Self {
        let settings_row = load_widget::<CenterBox>("NetworkSettingsRow");
        let exit_row = load_widget::<CenterBox>("NetworkExitRow");

        set_on_click(settings_row, |_| {
            gtk4::glib::spawn_future_local(async move {
                toggle_window("Networks");
                exec_async(&["kitty", "--name", "nmtui", "nmtui"]).await;
            });
        });

        set_on_click(exit_row, |_| {
            toggle_window("Networks");
        });

        for (idx, row) in rows().iter().enumerate() {
            set_on_click(row, move |label| {
                if idx >= State::list().len() {
                    return;
                }
                let network = &State::list()[idx];
                copy_to_clipboard(&network.ip);
                label.set_label("Copied!");
                gtk4::glib::spawn_future_local(async move {
                    gtk4::glib::timeout_future_seconds(1).await;
                    label.set_label(&format!("{}: {}", network.name, network.ip));
                });
            });
        }

        gtk4::glib::spawn_future_local(async move {
            let networks = all_networks().await;
            State::set(State(networks));

            sync_ui();
        });

        Self {
            reset: Box::new(sync_ui),
        }
    }
}
