use crate::{globals::load_widget, layers::Htop, utils::ToggleWindow};
use gtk4::{prelude::ButtonExt, Button};

pub(crate) fn init() {
    let widget = load_widget::<Button>("HtopWidget");

    widget.connect_clicked(|_| {
        Htop::toggle();
    });
}
