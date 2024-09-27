use gtk4::{prelude::WidgetExt, Label};

use crate::{globals::load_widget, models::Time};

pub(crate) fn init(format: &'static str, tooltip_format: &'static str) {
    let label = load_widget::<Label>("ClockWidgetLabel");

    Time::subscribe(|now| {
        label.set_label(&now.format(format).to_string());
        label.set_tooltip_text(Some(&now.format(tooltip_format).to_string()));
    });
}
