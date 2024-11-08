use crate::widgets::{
    Daily1, Daily2, Daily3, Daily4, Daily5, Daily6, Hourly1, Hourly10, Hourly2, Hourly3, Hourly4,
    Hourly5, Hourly6, Hourly7, Hourly8, Hourly9,
};
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
        Hourly1(),
        Hourly2(),
        Hourly3(),
        Hourly4(),
        Hourly5(),
        Hourly6(),
        Hourly7(),
        Hourly8(),
        Hourly9(),
        Hourly10(),
    ]
}

fn daily_labels() -> [&'static Label; 6] {
    [Daily1(), Daily2(), Daily3(), Daily4(), Daily5(), Daily6()]
}
