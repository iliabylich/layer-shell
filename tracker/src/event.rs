use crate::View;

#[derive(Debug)]
pub enum TrackerEvent {
    Updated(TrackerUpdatedEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct TrackerUpdatedEvent {
    pub view: View,
}
