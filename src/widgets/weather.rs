use gtk4::{prelude::ButtonExt, Button, Label};

use crate::{
    globals::{load_widget, toggle_window},
    models::WeatherApi,
    utils::TypedChildren,
};

pub(crate) struct Weather;

impl Weather {
    pub(crate) fn init() {
        let button = load_widget::<Button>("WeatherWidget");
        let [label] = button.children_as::<1, Label>();

        button.connect_clicked(|_| {
            toggle_window("Weather");
        });

        WeatherApi::subscribe(|weather| {
            label.set_label(&format!(
                "{} {}",
                weather.current.temperature,
                weather.current.code.icon(),
            ));
        });
    }
}
