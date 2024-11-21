use crate::{
    layers::weather::codes::{weather_code_to_description, weather_code_to_icon},
    widgets::weather::{daily_images, daily_labels, hourly_images, hourly_labels},
};
use gtk4::prelude::WidgetExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(|event| {
        if let Event::WeatherForecast { daily, hourly } = event {
            for ((label, image), weather) in hourly_labels().iter().zip(hourly_images()).zip(hourly)
            {
                let text = format!("{}' {:>5.1}℃", weather.hour, weather.temperature,);
                label.set_label(&text);
                label.set_tooltip_text(Some(&weather_code_to_description(weather.code)));

                image.set_from_gicon(weather_code_to_icon(weather.code));
            }

            for ((label, image), weather) in daily_labels().iter().zip(daily_images()).zip(daily) {
                let text = format!(
                    "{}: {:>5.1}℃ - {:>5.1}℃",
                    weather.day, weather.temperature.start, weather.temperature.end,
                );
                label.set_label(&text);
                label.set_tooltip_text(Some(&weather_code_to_description(weather.code)));

                image.set_from_gicon(weather_code_to_icon(weather.code));
            }
        }
    });
}
