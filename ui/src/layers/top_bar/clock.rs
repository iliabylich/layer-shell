use crate::widgets::ClockWidgetLabel;
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::Time { time, date } = event {
            ClockWidgetLabel().set_label(time);
            ClockWidgetLabel().set_tooltip_text(Some(date));
        }
    });
}
