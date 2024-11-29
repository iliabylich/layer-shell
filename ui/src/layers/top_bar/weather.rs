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
        if let Event::WeatherCurrent { temperature, code } = event {
            let label = format!("{}℃ {}", temperature, weather_code_to_description(*code));
            Label().set_label(&label);
        }
    });
}
