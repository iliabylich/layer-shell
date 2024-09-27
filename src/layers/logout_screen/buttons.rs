use crate::{globals::load_widget, layers::LogoutScreen, models::Logout, utils::LayerWindow};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    let lock_button = load_widget::<Button>("LogoutScreenLockButton");
    let reboot_button = load_widget::<Button>("LogoutScreenRebootButton");
    let shutdown_button = load_widget::<Button>("LogoutScreenShutdownButton");
    let logout_button = load_widget::<Button>("LogoutScreenLogoutButton");

    lock_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::lock();
    });

    reboot_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::reboot();
    });

    shutdown_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::shutdown();
    });

    logout_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::logout();
    });

    let buttons = [lock_button, reboot_button, shutdown_button, logout_button];

    Logout::subscribe(buttons.len(), move |active_idx| {
        for (idx, button) in buttons.iter().enumerate() {
            if idx == active_idx {
                button.add_css_class("widget-logout-button-action");
            } else {
                button.remove_css_class("widget-logout-button-action");
            }
        }
    });

    (
        Box::new(Logout::reset),
        Box::new(|key| match key {
            "Left" => Logout::left(),
            "Right" => Logout::right(),
            _ => {}
        }),
    )
}
