use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};

widget!(Window, gtk4::Window);
const HOURS: usize = 10;
widget!(HourlyRows, [(gtk4::Label, gtk4::Image); HOURS]);
const DAYS: usize = 6;
widget!(DailyRows, [(gtk4::Label, gtk4::Image); DAYS]);

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("WeatherWindow");
    window.set_css_classes(&["widget-weather"]);

    let layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    window.set_child(Some(&layout));

    let left_side = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    left_side.add_css_class("weather-left-side");
    layout.append(&left_side);

    left_side.append(&gtk4::Label::new(Some("Hourly")));
    let hourly_rows = std::array::from_fn::<_, HOURS, _>(|_| {
        let (row, label, image) = weather_row();
        left_side.append(&row);
        (label, image)
    });

    let right_side = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    right_side.add_css_class("weather-right-side");
    layout.append(&right_side);

    right_side.append(&gtk4::Label::new(Some("Daily")));
    let daily_rows = std::array::from_fn::<_, DAYS, _>(|_| {
        let (row, label, image) = weather_row();
        right_side.append(&row);
        (label, image)
    });

    set_Window(window);
    set_HourlyRows(hourly_rows);
    set_DailyRows(daily_rows);
}

fn weather_row() -> (gtk4::Box, gtk4::Label, gtk4::Image) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let label = gtk4::Label::new(Some("..."));
    let image = gtk4::Image::new();
    image.set_pixel_size(24);
    row.append(&label);
    row.append(&image);
    (row, label, image)
}
