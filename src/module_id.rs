#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModuleId {
    KbMod,
    NM,
    PW,
    Niri,
    Weather,
    Tray,
    Control,
    Cpu,
    Memory,
    Timer,
}

impl ModuleId {
    pub(crate) const MODULES_COUNT: usize = 10;

    pub(crate) fn new(value: u64) -> Self {
        if value == Self::Weather as u64 {
            Self::Weather
        } else if value == Self::KbMod as u64 {
            Self::KbMod
        } else if value == Self::NM as u64 {
            Self::NM
        } else if value == Self::PW as u64 {
            Self::PW
        } else if value == Self::Niri as u64 {
            Self::Niri
        } else if value == Self::Tray as u64 {
            Self::Tray
        } else if value == Self::Control as u64 {
            Self::Control
        } else if value == Self::Cpu as u64 {
            Self::Cpu
        } else if value == Self::Memory as u64 {
            Self::Memory
        } else if value == Self::Timer as u64 {
            Self::Timer
        } else {
            panic!("can't build ModuleId from {value}")
        }
    }
}
