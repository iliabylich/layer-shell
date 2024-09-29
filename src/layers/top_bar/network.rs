use crate::{globals::load_widget, layers::Networks, models::WiFiStatus, utils::LayerWindow};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    let label = load_widget::<Label>("NetworkWidgetLabel");

    WiFiStatus::spawn(|status| {
        if let Some(status) = status {
            label.set_label(&format!("{} ({})% ", status.ssid, status.strength));
        } else {
            label.set_label("Not connected");
        }
    });

    let button = load_widget::<Button>("NetworkWidget");
    button.connect_clicked(|_| {
        Networks::toggle();
    });
}
