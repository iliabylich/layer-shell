use gtk4::{prelude::WidgetExt, Label};

use crate::{globals::load_widget, models::Time};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn init(format: &'static str, tooltip_format: &'static str) {
        let label: &Label = load_widget("ClockLabel");

        Time::spawn(move |now| {
            label.set_label(&now.format(format).to_string());
            label.set_tooltip_text(Some(&now.format(tooltip_format).to_string()));
        });
    }
}
