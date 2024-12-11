use crate::{
    layers::{weather::codes::weather_code_to_description, Weather},
    widgets::top_bar::weather::{Label, Widget},
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    Widget().connect_clicked(|_| {
        Weather::toggle();
    });

    subscribe(|event| {
        if let Event::CurrentWeather(event) = event {
            let label = format!(
                "{}℃ {}",
                event.temperature,
                weather_code_to_description(event.code)
            );
            Label().set_label(&label);
        }
    });
}
