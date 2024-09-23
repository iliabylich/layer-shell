use crate::{globals::load_widget, layers::LogoutScreen, models::Logout, utils::ToggleWindow};
use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

pub(crate) fn init() -> (Box<dyn Fn()>, Box<dyn Fn(&str)>) {
    let buttons = [
        load_widget::<Button>("LockButton"),
        load_widget::<Button>("RebootButton"),
        load_widget::<Button>("ShutdownButton"),
        load_widget::<Button>("LogoutButton"),
    ];

    buttons[0].connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::lock();
    });

    buttons[1].connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::reboot();
    });

    buttons[2].connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::shutdown();
    });

    buttons[3].connect_clicked(|_| {
        LogoutScreen::toggle();
        Logout::logout();
    });

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
