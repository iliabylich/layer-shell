use gtk4::{prelude::ButtonExt, Button};

use crate::globals::{load_widget, toggle_window};

pub(crate) struct PowerButton;

impl PowerButton {
    pub(crate) fn init() {
        let widget = load_widget::<Button>("PowerButton");

        widget.connect_clicked(|_| {
            toggle_window("LogoutScreen");
        });
    }
}
