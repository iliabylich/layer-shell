use crate::{layers::SessionScreen, widgets::top_bar::session::Widget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    Widget().connect_clicked(|_| {
        SessionScreen::toggle();
    });
}
