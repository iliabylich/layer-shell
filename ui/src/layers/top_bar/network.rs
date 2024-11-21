use crate::{
    icons::wifi_icon,
    layers::Networks,
    widgets::{NetworkWidget, NetworkWidgetImage, NetworkWidgetLabel},
};
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_io::{subscribe, Event, WiFiStatus};

pub(crate) fn init() {
    NetworkWidgetImage().set_from_gicon(wifi_icon());

    subscribe(|event| {
        if let Event::WiFiStatus(status) = event {
            if let Some(WiFiStatus { ssid, strength }) = status {
                NetworkWidgetLabel().set_label(&format!("{} ({})% ", ssid, strength));
                NetworkWidgetImage().set_visible(true);
            } else {
                NetworkWidgetLabel().set_label("Not connected");
                NetworkWidgetImage().set_visible(false);
            }
        }
    });

    NetworkWidget().connect_clicked(|_| {
        Networks::toggle();
    });
}
