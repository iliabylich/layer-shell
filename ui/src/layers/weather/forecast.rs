use crate::widgets;
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::WeatherForecast { daily, hourly } = event {
            for (label, (text, tooltip)) in widgets::weather::hourly_labels().iter().zip(hourly) {
                label.set_label(text);
                label.set_tooltip_text(Some(tooltip));
            }

            for (label, (text, tooltip)) in widgets::weather::daily_labels().iter().zip(daily) {
                label.set_label(text);
                label.set_tooltip_text(Some(tooltip));
            }
        }
    });
}
