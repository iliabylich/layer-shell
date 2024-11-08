use crate::{layers::Htop, widgets::HtopWidget};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    HtopWidget().connect_clicked(|_| {
        Htop::toggle();
    });
}
