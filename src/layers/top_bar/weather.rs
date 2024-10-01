use crate::{
    globals::load_widget,
    layers::Weather,
    models::{subscribe, Event},
    utils::LayerWindow,
};
use gtk4::{prelude::ButtonExt, Button, Label};

pub(crate) fn init() {
    let button = load_widget::<Button>("WeatherWidget");
    button.connect_clicked(|_| {
        Weather::toggle();
    });

    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::WeatherCurrent(weather) = event {
        let label = load_widget::<Label>("WeatherWidgetLabel");
        label.set_label(weather);
    }
}
