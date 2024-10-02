use crate::{globals::load_widget, layers::LogoutScreen, utils::LayerWindow};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    let [lock_button, reboot_button, shutdown_button, logout_button] = buttons();
    lock_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Lock);
    });

    reboot_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Reboot);
    });

    shutdown_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Shutdown);
    });

    logout_button.connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Logout);
    });

    subscribe(on_event);

    (
        Box::new(|| publish(Command::SessionReset)),
        Box::new(|key| match key {
            "Left" => publish(Command::SessionGoLeft),
            "Right" => publish(Command::SessionGoRight),
            _ => {}
        }),
    )
}

fn on_event(event: &Event) {
    if let Event::SessionScreen(active_idx) = event {
        for (idx, button) in buttons().iter().enumerate() {
            if idx == *active_idx {
                button.add_css_class("widget-logout-button-action");
            } else {
                button.remove_css_class("widget-logout-button-action");
            }
        }
    }
}

fn buttons() -> [&'static Button; 4] {
    let lock_button = load_widget::<Button>("LogoutScreenLockButton");
    let reboot_button = load_widget::<Button>("LogoutScreenRebootButton");
    let shutdown_button = load_widget::<Button>("LogoutScreenShutdownButton");
    let logout_button = load_widget::<Button>("LogoutScreenLogoutButton");
    [lock_button, reboot_button, shutdown_button, logout_button]
}
