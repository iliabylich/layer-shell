mod writer;

mod reader;

mod event;
pub use event::Event;

mod stream;
pub use stream::HyprlandStream as Hyprland;

mod env;
mod state;
