use crate::{
    layers::LogoutScreen,
    widgets::{
        LogoutScreenLockButton, LogoutScreenLogoutButton, LogoutScreenRebootButton,
        LogoutScreenShutdownButton,
    },
};
use gtk4::prelude::{ButtonExt, WidgetExt};
use layer_shell_io::{publish, subscribe, Command, Event};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    LogoutScreenLockButton().connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Lock);
    });

    LogoutScreenRebootButton().connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Reboot);
    });

    LogoutScreenShutdownButton().connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Shutdown);
    });

    LogoutScreenLockButton().connect_clicked(|_| {
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
        let buttons = [
            LogoutScreenLockButton(),
            LogoutScreenRebootButton(),
            LogoutScreenShutdownButton(),
            LogoutScreenLogoutButton(),
        ];

        for (idx, button) in buttons.iter().enumerate() {
            if idx == *active_idx {
                button.add_css_class("widget-logout-button-action");
            } else {
                button.remove_css_class("widget-logout-button-action");
            }
        }
    }
}
