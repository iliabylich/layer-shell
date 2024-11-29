use crate::{
    icons::power_icon,
    layers::SessionScreen,
    widgets::top_bar::session::{Image, Widget},
};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    Image().set_from_gicon(power_icon());

    Widget().connect_clicked(|_| {
        SessionScreen::toggle();
    });
}
