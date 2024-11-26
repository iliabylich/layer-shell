#[derive(Clone, Copy, Debug)]
pub enum Event {
    MuteChanged(bool),
    VolumeChanged(f32),
}
