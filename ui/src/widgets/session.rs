use crate::widgets::widget;
use gtk4::prelude::{BoxExt, GtkWindowExt, WidgetExt};
use vte4::ButtonExt;

widget!(Window, gtk4::Window);
widget!(LockButton, gtk4::Button);
widget!(RebootButton, gtk4::Button);
widget!(ShutdownButton, gtk4::Button);
widget!(LogoutButton, gtk4::Button);

pub(crate) fn setup() {
    let window = gtk4::Window::new();
    window.set_widget_name("SessionWindow");
    window.add_css_class("session-window");

    let layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    layout.set_homogeneous(true);
    layout.set_spacing(200);
    layout.add_css_class("session-window-wrapper");
    window.set_child(Some(&layout));

    let lock = button("Lock");
    layout.append(&lock);

    let reboot = button("Reboot");
    layout.append(&reboot);

    let shutdown = button("Shutdown");
    layout.append(&shutdown);

    let logout = button("Logout");
    layout.append(&logout);

    set_Window(window);
    set_LockButton(lock);
    set_RebootButton(reboot);
    set_ShutdownButton(shutdown);
    set_LogoutButton(logout);
}

fn button(text: &str) -> gtk4::Button {
    let button = gtk4::Button::new();
    button.add_css_class("session-window-button");

    let label = gtk4::Label::new(Some(text));
    button.set_child(Some(&label));

    button
}
