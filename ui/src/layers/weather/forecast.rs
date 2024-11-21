use crate::{
    layers::weather::codes::{weather_code_to_description, weather_code_to_icon},
    widgets,
};
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::WeatherForecast { daily, hourly } = event {
            for (label, weather) in widgets::weather::hourly_labels().iter().zip(hourly) {
                let text = format!(
                    "{}' {:>5.1}℃ {}",
                    weather.hour,
                    weather.temperature,
                    weather_code_to_icon(weather.code)
                );
                label.set_label(&text);

                label.set_tooltip_text(Some(&weather_code_to_description(weather.code)));
            }

            for (label, weather) in widgets::weather::daily_labels().iter().zip(daily) {
                let text = format!(
                    "{}: {:>5.1}℃ - {:>5.1}℃ {}",
                    weather.day,
                    weather.temperature.start,
                    weather.temperature.end,
                    weather_code_to_icon(weather.code)
                );
                label.set_label(&text);

                label.set_tooltip_text(Some(&weather_code_to_description(weather.code)));
            }
        }
    });
}
