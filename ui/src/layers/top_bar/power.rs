use crate::{layers::LogoutScreen, widgets::PowerWidget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    PowerWidget().connect_clicked(|_| {
        LogoutScreen::toggle();
    });
}
