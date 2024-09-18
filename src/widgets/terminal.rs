use gtk4::Button;
use vte4::ButtonExt;

use crate::globals::{load_widget, toggle_window};

pub(crate) struct Terminal;

impl Terminal {
    pub(crate) fn init() {
        let widget = load_widget::<Button>("Terminal");

        widget.connect_clicked(|_| {
            toggle_window("Htop");
        });
    }
}
