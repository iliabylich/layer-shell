use crate::{
    layers::Networks,
    widgets::{NetworkWidget, NetworkWidgetLabel},
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{subscribe, Event, WiFiStatus};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::WiFiStatus(status) = event {
            if let Some(WiFiStatus { ssid, strength }) = status {
                NetworkWidgetLabel().set_label(&format!("{} ({})% ", ssid, strength));
            } else {
                NetworkWidgetLabel().set_label("Not connected");
            }
        }
    });

    NetworkWidget().connect_clicked(|_| {
        Networks::toggle();
    });
}
