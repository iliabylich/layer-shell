#[derive(Debug)]
pub enum SoundEvent {
    InitialSoundEvent(InitialSoundEvent),
    VolumeChangedEvent(VolumeChangedEvent),
    MuteChangedEvent(MuteChangedEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct InitialSoundEvent {
    pub volume: u32,
    pub muted: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct VolumeChangedEvent {
    pub volume: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MuteChangedEvent {
    pub muted: bool,
}
