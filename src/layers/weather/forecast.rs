use gtk4::{prelude::WidgetExt, Label};

use crate::{globals::load_widget, models::WeatherApi};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    (Box::new(sync_ui), Box::new(|_key| {}))
}

fn sync_ui() {
    if let Some(weather) = WeatherApi::get_cached() {
        let now = chrono::Local::now().naive_local();
        let matching_hours = weather.hourly.iter().filter(|hourly| hourly.hour > now);
        for (label, weather) in hourly_labels().iter().zip(matching_hours) {
            label.set_label(&format!(
                "{}' {} {}",
                weather.hour.format("%H"),
                weather.temperature,
                weather.code.icon()
            ));
            label.set_tooltip_text(Some(&format!("{}", weather.code)));
        }

        let today = chrono::Local::now().date_naive();
        let matching_days = weather.daily.iter().filter(|daily| daily.day > today);
        for (label, weather) in daily_labels().iter().zip(matching_days) {
            label.set_label(&format!(
                "{}: {:.1} - {:.1} {}",
                weather.day.format("%m-%d"),
                weather.temperature_min,
                weather.temperature_max,
                weather.code.icon()
            ));
            label.set_tooltip_text(Some(&format!("{}", weather.code)));
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
