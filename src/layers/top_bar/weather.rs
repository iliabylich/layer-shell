use gtk4::{prelude::ButtonExt, Button, Label};

use crate::{
    globals::load_widget,
    layers::Weather,
    models::WeatherApi,
    utils::{LayerWindow, TypedChildren},
};

pub(crate) fn init() {
    let button = load_widget::<Button>("WeatherWidget");
    let [label] = button.children_as::<1, Label>();

    button.connect_clicked(|_| {
        Weather::toggle();
    });

    WeatherApi::subscribe(|weather| {
        label.set_label(&format!(
            "{} {}",
            weather.current.temperature,
            weather.current.code.icon(),
        ));
    });
}
