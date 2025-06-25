mod env;
mod event;
mod hyprland;
mod reader;
mod state;
mod writer;

pub use event::{HyprlandEvent, LanguageEvent, WorkspacesEvent};
pub use hyprland::Hyprland;

#[macro_export]
macro_rules! hyprctl {
    ($($arg:tt)*) => {
        if let Err(err) = $crate::Hyprland::hyprctl_dispatch(format!($($arg)*)).await {
            log::error!("{err:?}");
        }
    };
}
