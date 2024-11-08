use crate::{
    layers::Weather,
    widgets::{WeatherWidget, WeatherWidgetLabel},
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    WeatherWidget().connect_clicked(|_| {
        Weather::toggle();
    });

    subscribe(|event| {
        if let Event::WeatherCurrent(weather) = event {
            WeatherWidgetLabel().set_label(weather);
        }
    });
}
