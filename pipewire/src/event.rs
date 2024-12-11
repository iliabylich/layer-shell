#[derive(Debug)]
pub enum Event {
    Volume(Volume),
}

#[derive(Debug)]
pub struct Volume {
    pub volume: f32,
}
