use crate::{
    layers::SessionScreen,
    widgets::session::{LockButton, LogoutButton, RebootButton, ShutdownButton},
};
use gtk4::prelude::ButtonExt;
use layer_shell_io::{publish, Command};

pub(crate) fn init() {
    LockButton().connect_clicked(|_| {
        SessionScreen::toggle();
        publish(Command::Lock);
    });

    RebootButton().connect_clicked(|_| {
        SessionScreen::toggle();
        publish(Command::Reboot);
    });

    ShutdownButton().connect_clicked(|_| {
        SessionScreen::toggle();
        publish(Command::Shutdown);
    });

    LogoutButton().connect_clicked(|_| {
        SessionScreen::toggle();
        publish(Command::Logout);
    });
}
