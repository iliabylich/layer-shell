use crate::{
    layers::LogoutScreen,
    widgets::{power_icon, PowerWidget, PowerWidgetImage},
};
use gtk4::prelude::ButtonExt;

pub(crate) fn init() {
    PowerWidgetImage().set_from_gicon(power_icon());

    PowerWidget().connect_clicked(|_| {
        LogoutScreen::toggle();
    });
}
