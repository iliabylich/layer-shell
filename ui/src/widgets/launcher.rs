use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};

widget!(Window, gtk4::Window);
widget!(Input, gtk4::SearchEntry);
const ROWS: usize = 5;
widget!(Rows, [(gtk4::Box, gtk4::Image, gtk4::Label); ROWS]);

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("LauncherWindow");
    window.set_width_request(700);

    let layout = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    layout.add_css_class("widget-launcher-wrapper");
    window.set_child(Some(&layout));

    let input = gtk4::SearchEntry::new();
    input.add_css_class("widget-launcher-search-box");
    input.set_hexpand(true);
    layout.append(&input);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.add_css_class("widget-launcher-scroll-list");
    scroll.set_can_focus(false);
    layout.append(&scroll);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    scroll.set_child(Some(&content));

    let rows = std::array::from_fn::<_, ROWS, _>(|_| {
        let (row, image, label) = row();
        content.append(&row);
        (row, image, label)
    });

    set_Window(window);
    set_Input(input);
    set_Rows(rows);
}

fn row() -> (gtk4::Box, gtk4::Image, gtk4::Label) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    row.add_css_class("widget-launcher-row");

    let image = gtk4::Image::new();
    image.set_icon_size(gtk4::IconSize::Large);

    let label = gtk4::Label::new(Some("..."));
    label.set_xalign(0.0);
    label.set_valign(gtk4::Align::Center);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    row.append(&image);
    row.append(&label);

    (row, image, label)
}
