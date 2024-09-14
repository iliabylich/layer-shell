use gtk4::{
    prelude::{ButtonExt, WidgetExt},
    Button,
};

use crate::{
    globals::{load_widget, toggle_window},
    models::Logout as LogoutModel,
};

pub(crate) struct Logout {
    pub(crate) reset: Box<dyn Fn()>,
    pub(crate) on_key_press: Box<dyn Fn(&str)>,
}

impl Logout {
    pub(crate) fn init() -> Self {
        let buttons = [
            load_widget::<Button>("LockButton"),
            load_widget::<Button>("RebootButton"),
            load_widget::<Button>("ShutdownButton"),
            load_widget::<Button>("LogoutButton"),
        ];

        buttons[0].connect_clicked(|_| {
            toggle_window("LogoutScreen");
            LogoutModel::lock();
        });

        buttons[1].connect_clicked(|_| {
            toggle_window("LogoutScreen");
            LogoutModel::reboot();
        });

        buttons[2].connect_clicked(|_| {
            toggle_window("LogoutScreen");
            LogoutModel::shutdown();
        });

        buttons[3].connect_clicked(|_| {
            toggle_window("LogoutScreen");
            LogoutModel::logout();
        });

        LogoutModel::spawn(buttons.len(), move |active_idx| {
            for (idx, button) in buttons.iter().enumerate() {
                if idx == active_idx {
                    button.add_css_class("widget-logout-button-action");
                } else {
                    button.remove_css_class("widget-logout-button-action");
                }
            }
        });

        Self {
            reset: Box::new(LogoutModel::reset),
            on_key_press: Box::new(|key| match key {
                "Left" => LogoutModel::left(),
                "Right" => LogoutModel::right(),
                _ => {}
            }),
        }
    }
}
