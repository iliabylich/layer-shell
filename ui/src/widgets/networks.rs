use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};
use vte4::OrientableExt;

widget!(Window, gtk4::Window);
const ROWS: usize = 5;
widget!(Rows, [(gtk4::CenterBox, gtk4::Label); ROWS]);
widget!(SettingsRow, gtk4::CenterBox);
widget!(ExitRow, gtk4::CenterBox);

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("NetworksWindow");
    window.set_width_request(700);

    let layout = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    layout.add_css_class("widget-network-row-list");
    window.set_child(Some(&layout));

    let rows = std::array::from_fn::<_, ROWS, _>(|_| {
        let (row, label) = row(None, "edit-copy");
        layout.append(&row);
        (row, label)
    });

    let (settings_row, _) = row(Some("Settings (nmtui)"), "preferences-system-network");
    layout.append(&settings_row);

    let (exit_row, _) = row(Some("Close"), "window-close");
    layout.append(&exit_row);

    set_Window(window);
    set_Rows(rows);
    set_SettingsRow(settings_row);
    set_ExitRow(exit_row);
}

fn row(text: Option<&str>, icon: &str) -> (gtk4::CenterBox, gtk4::Label) {
    let row = gtk4::CenterBox::new();
    row.add_css_class("widget-network-row");
    row.set_orientation(gtk4::Orientation::Horizontal);
    row.set_halign(gtk4::Align::Fill);

    let label = gtk4::Label::new(None);
    label.set_justify(gtk4::Justification::Left);
    label.set_xalign(0.0);
    if let Some(text) = text {
        label.set_label(text);
    }
    row.set_start_widget(Some(&label));

    let image = gtk4::Image::new();
    image.set_icon_name(Some(icon));
    image.set_icon_size(gtk4::IconSize::Large);
    image.set_pixel_size(30);
    row.set_end_widget(Some(&image));

    (row, label)
}
