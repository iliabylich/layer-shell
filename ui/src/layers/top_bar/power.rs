use crate::{layers::LogoutScreen, utils::LayerWindow, widgets::PowerWidget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    PowerWidget().connect_clicked(|_| {
        LogoutScreen::toggle();
    });
}
