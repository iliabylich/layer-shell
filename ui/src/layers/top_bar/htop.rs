use crate::{layers::Htop, widgets::top_bar::htop::Widget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    Widget().connect_clicked(|_| {
        Htop::toggle();
    });
}
