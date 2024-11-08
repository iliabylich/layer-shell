use crate::{layers::Htop, utils::LayerWindow, widgets::HtopWidget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    HtopWidget().connect_clicked(|_| {
        Htop::toggle();
    });
}
