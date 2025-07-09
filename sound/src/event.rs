#[derive(Debug)]
pub enum SoundEvent {
    MuteChangedEvent(MuteChangedEvent),
    VolumeChangedEvent(VolumeChangedEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct MuteChangedEvent {
    pub muted: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct VolumeChangedEvent {
    pub volume: u32,
}
