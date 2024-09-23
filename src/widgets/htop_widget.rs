use gtk4::Button;
use vte4::ButtonExt;

use crate::globals::{load_widget, toggle_window};

pub(crate) struct HtopWidget;

impl HtopWidget {
    pub(crate) fn init() {
        let widget = load_widget::<Button>("HtopWidget");

        widget.connect_clicked(|_| {
            toggle_window("Htop");
        });
    }
}
