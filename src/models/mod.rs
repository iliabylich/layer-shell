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
