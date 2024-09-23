use crate::{globals::load_widget, layers::LogoutScreen, utils::ToggleWindow};
use gtk4::{prelude::ButtonExt, Button};

pub(crate) fn init() {
    let widget = load_widget::<Button>("PowerWidget");

    widget.connect_clicked(|_| {
        LogoutScreen::toggle();
    });
}
