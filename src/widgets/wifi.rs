use gtk4::{prelude::ButtonExt, Button, Label};

use crate::{
    globals::{load_widget, toggle_window},
    models::WiFiStatus,
};

pub(crate) struct WiFi;

impl WiFi {
    pub(crate) fn init() {
        let widget = load_widget::<Button>("WiFi");
        let label = load_widget::<Label>("WiFiLabel");

        WiFiStatus::spawn(|status| {
            if let Some(status) = status {
                label.set_label(&format!("{} ({})% ", status.ssid, status.strength));
            } else {
                label.set_label("Not connected");
            }
        });

        widget.connect_clicked(|_| {
            toggle_window("Networks");
        });
    }
}
