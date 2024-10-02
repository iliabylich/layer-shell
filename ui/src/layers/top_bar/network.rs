use crate::{globals::load_widget, layers::Networks, utils::LayerWindow};
use gtk4::{prelude::ButtonExt, Button, Label};
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);

    let button = load_widget::<Button>("NetworkWidget");
    button.connect_clicked(|_| {
        Networks::toggle();
    });
}

fn on_event(event: &Event) {
    if let Event::WiFi(state) = event {
        let label = load_widget::<Label>("NetworkWidgetLabel");

        if let Some((ssid, strength)) = state {
            label.set_label(&format!("{} ({})% ", ssid, strength));
        } else {
            label.set_label("Not connected");
        }
    }
}
