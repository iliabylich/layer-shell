#[derive(Debug)]
#[repr(C)]
pub struct MemoryEvent {
    pub used: f64,
    pub total: f64,
}
