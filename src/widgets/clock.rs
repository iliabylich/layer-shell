use gtk4::{prelude::WidgetExt, CenterBox, Label};

use crate::{globals::load_widget, models::Time, utils::TypedChildren};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn init(format: &'static str, tooltip_format: &'static str) {
        let label = load_widget::<CenterBox>("ClockWidget").first_child_as::<Label>();

        Time::spawn(|now| {
            label.set_label(&now.format(format).to_string());
            label.set_tooltip_text(Some(&now.format(tooltip_format).to_string()));
        });
    }
}
