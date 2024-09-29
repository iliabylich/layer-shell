use crate::{globals::load_widget, layers::Weather, models::WeatherApi, utils::LayerWindow};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    let button = load_widget::<Button>("WeatherWidget");
    button.connect_clicked(|_| {
        Weather::toggle();
    });

    WeatherApi::subscribe(|weather| {
        let label = load_widget::<Label>("WeatherWidgetLabel");
        label.set_label(&format!(
            "{} {}",
            weather.current.temperature,
            weather.current.code.icon(),
        ));
    });
}
