use crate::globals::load_widget;
use gtk4::{prelude::WidgetExt, Label};
use layer_shell_io::{subscribe, Event};

pub(crate) fn init() {
    subscribe(on_event);
}

fn on_event(event: &Event) {
    if let Event::WeatherForecast { daily, hourly } = event {
        for (label, (text, tooltip)) in hourly_labels().iter().zip(hourly) {
            label.set_label(text);
            label.set_tooltip_text(Some(tooltip));
        }

        for (label, (text, tooltip)) in daily_labels().iter().zip(daily) {
            label.set_label(text);
            label.set_tooltip_text(Some(tooltip));
        }
    }
}

fn hourly_labels() -> [&'static Label; 10] {
    [
        load_widget::<Label>("Hourly1"),
        load_widget::<Label>("Hourly2"),
        load_widget::<Label>("Hourly3"),
        load_widget::<Label>("Hourly4"),
        load_widget::<Label>("Hourly5"),
        load_widget::<Label>("Hourly6"),
        load_widget::<Label>("Hourly7"),
        load_widget::<Label>("Hourly8"),
        load_widget::<Label>("Hourly9"),
        load_widget::<Label>("Hourly10"),
    ]
}

fn daily_labels() -> [&'static Label; 6] {
    [
        load_widget::<Label>("Daily1"),
        load_widget::<Label>("Daily2"),
        load_widget::<Label>("Daily3"),
        load_widget::<Label>("Daily4"),
        load_widget::<Label>("Daily5"),
        load_widget::<Label>("Daily6"),
    ]
}
