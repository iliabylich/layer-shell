mod singleton;
pub(crate) use singleton::singleton;

mod hyprland;
pub(crate) use hyprland::{HyprlandLanguage, HyprlandWorkspaces};

mod cpu;
pub(crate) use cpu::CPU;

mod memory;
pub(crate) use memory::Memory;

mod time;
pub(crate) use time::Time;

mod output_sound;
pub(crate) use output_sound::OutputSound;

mod logout;
pub(crate) use logout::Logout;

mod network_manager;
pub(crate) use network_manager::{all_networks, Iface, WiFiStatus};
