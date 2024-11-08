use crate::{
    layers::Networks,
    utils::LayerWindow,
    widgets::{NetworkWidget, NetworkWidgetLabel},
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);

    NetworkWidget().connect_clicked(|_| {
        Networks::toggle();
    });
}

fn on_event(event: &Event) {
    if let Event::WiFi(state) = event {
        if let Some((ssid, strength)) = state {
            NetworkWidgetLabel().set_label(&format!("{} ({})% ", ssid, strength));
        } else {
            NetworkWidgetLabel().set_label("Not connected");
        }
    }
}
