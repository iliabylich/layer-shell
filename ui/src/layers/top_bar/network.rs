use crate::{
    layers::Networks,
    widgets::top_bar::network::{Image, Label, Widget},
};
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_io::{subscribe, Event, WiFiStatus};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::WiFiStatus(status) = event {
            if let Some(WiFiStatus { ssid, strength }) = status {
                Label().set_label(&format!("{} ({})% ", ssid, strength));
                Image().set_visible(true);
            } else {
                Label().set_label("Not connected");
                Image().set_visible(false);
            }
        }
    });

    Widget().connect_clicked(|_| {
        Networks::toggle();
    });
}
