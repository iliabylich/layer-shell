use crate::{
    globals::load_widget,
    layers::Networks,
    models::WiFiStatus,
    utils::{ToggleWindow, TypedChildren},
};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    let widget = load_widget::<Button>("NetworkWidget");
    let label = widget.first_child_as::<Label>();

    WiFiStatus::spawn(|status| {
        if let Some(status) = status {
            label.set_label(&format!("{} ({})% ", status.ssid, status.strength));
        } else {
            label.set_label("Not connected");
        }
    });

    widget.connect_clicked(|_| {
        Networks::toggle();
    });
}
