use crate::modules::{TrayIcon, TrayItem, tray::app::TrayEvent};
use alloc::vec::Vec;

pub(crate) enum State {
    Nothing,
    OnlyIcon(TrayIcon),
    OnlyLayout(Vec<TrayItem>),
    All,
}

impl State {
    pub(crate) fn on_icon_received(&mut self, new_icon: TrayIcon) -> Option<TrayEvent> {
        match self {
            Self::Nothing => {
                *self = Self::OnlyIcon(new_icon);
                None
            }
            Self::OnlyIcon(icon) => {
                *icon = new_icon;
                None
            }
            Self::OnlyLayout(layout) => {
                let layout = core::mem::take(layout);
                *self = Self::All;
                Some(TrayEvent::Initialized(new_icon, layout))
            }
            Self::All => Some(TrayEvent::IconUpdated(new_icon)),
        }
    }

    pub(crate) fn on_layout_receieved(&mut self, new_layout: Vec<TrayItem>) -> Option<TrayEvent> {
        match self {
            Self::Nothing => {
                *self = Self::OnlyLayout(new_layout);
                None
            }
            Self::OnlyIcon(icon) => {
                let icon = core::mem::take(icon);
                *self = Self::All;
                Some(TrayEvent::Initialized(icon, new_layout))
            }
            Self::OnlyLayout(layout) => {
                *layout = new_layout;
                None
            }
            Self::All => Some(TrayEvent::MenuUpdated(new_layout)),
        }
    }
}
