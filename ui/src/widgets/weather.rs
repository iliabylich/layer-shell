use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};
use std::mem::MaybeUninit;

widget!(Window, gtk4::Window);
const HOURS: usize = 10;
widget!(HourlyLabels, [gtk4::Label; HOURS]);
widget!(HourlyImages, [gtk4::Image; HOURS]);
const DAYS: usize = 6;
widget!(DailyLabels, [gtk4::Label; DAYS]);
widget!(DailyImages, [gtk4::Image; DAYS]);

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
    let mut hourly_images: [MaybeUninit<gtk4::Image>; HOURS] =
        [const { MaybeUninit::uninit() }; HOURS];
    let mut hourly_labels: [MaybeUninit<gtk4::Label>; HOURS] =
        [const { MaybeUninit::uninit() }; HOURS];
    for i in 0..HOURS {
        let (row, label, image) = weather_row();
        left_side.append(&row);
        hourly_labels[i].write(label);
        hourly_images[i].write(image);
    }

    let right_side = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    right_side.add_css_class("weather-right-side");
    layout.append(&right_side);

    right_side.append(&gtk4::Label::new(Some("Daily")));
    let mut daily_images: [MaybeUninit<gtk4::Image>; DAYS] =
        [const { MaybeUninit::uninit() }; DAYS];
    let mut daily_labels: [MaybeUninit<gtk4::Label>; DAYS] =
        [const { MaybeUninit::uninit() }; DAYS];
    for i in 0..DAYS {
        let (row, label, image) = weather_row();
        right_side.append(&row);
        daily_labels[i].write(label);
        daily_images[i].write(image);
    }

    set_Window(window);
    set_HourlyLabels(unsafe {
        std::mem::transmute::<[MaybeUninit<gtk4::Label>; HOURS], [gtk4::Label; HOURS]>(
            hourly_labels,
        )
    });
    set_HourlyImages(unsafe {
        std::mem::transmute::<[MaybeUninit<gtk4::Image>; HOURS], [gtk4::Image; HOURS]>(
            hourly_images,
        )
    });
    set_DailyLabels(unsafe {
        std::mem::transmute::<[MaybeUninit<gtk4::Label>; DAYS], [gtk4::Label; DAYS]>(daily_labels)
    });
    set_DailyImages(unsafe {
        std::mem::transmute::<[MaybeUninit<gtk4::Image>; DAYS], [gtk4::Image; DAYS]>(daily_images)
    });
}

fn weather_row() -> (gtk4::Box, gtk4::Label, gtk4::Image) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let label = gtk4::Label::new(None);
    let image = gtk4::Image::new();
    image.set_pixel_size(24);
    row.append(&label);
    row.append(&image);
    (row, label, image)
}
