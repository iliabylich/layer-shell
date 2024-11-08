use crate::{
    layers::LogoutScreen,
    widgets::{
        LogoutScreenLockButton, LogoutScreenLogoutButton, LogoutScreenRebootButton,
        LogoutScreenShutdownButton,
    },
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{publish, Command};

pub(crate) fn init() {
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

    LogoutScreenLogoutButton().connect_clicked(|_| {
        LogoutScreen::toggle();
        publish(Command::Logout);
    });
}
