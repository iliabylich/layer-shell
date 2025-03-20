use crate::hyprctl;

pub(crate) struct Session;

impl Session {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn lock(&mut self) {
        if let Err(err) = hyprctl::dispatch("exec hyprlock") {
            log::error!("failed to lock: {:?}", err);
        }
    }
    pub(crate) fn reboot(&mut self) {
        if let Err(err) = hyprctl::dispatch("exec systemctl reboot") {
            log::error!("failed to reboot: {:?}", err);
        }
    }
    pub(crate) fn shutdown(&mut self) {
        if let Err(err) = hyprctl::dispatch("exec systemctl poweroff") {
            log::error!("failed to shutdown: {:?}", err);
        }
    }
    pub(crate) fn logout(&mut self) {
        if let Err(err) = hyprctl::dispatch("exit") {
            log::error!("failed to logout: {:?}", err);
        }
    }
    pub(crate) fn change_theme(&mut self) {
        if let Err(err) = hyprctl::dispatch("exec ~/.config/hypr/wallpaper-change.sh") {
            log::error!("failed to change theme: {:?}", err);
        }
    }
}
