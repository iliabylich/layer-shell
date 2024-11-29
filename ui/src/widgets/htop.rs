use crate::widgets::widget;
use gtk4::prelude::WidgetExt;

widget!(Window, gtk4::Window);

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("HtopWindow");
    window.add_css_class("widget-htop");
    window.set_width_request(1000);
    window.set_height_request(700);

    set_Window(window);
}
