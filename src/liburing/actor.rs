use crate::Event;

use super::{IoUring, cqe::Cqe, pending::Pending};
use anyhow::Result;

pub(crate) trait Actor {
    fn drain_once(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
        events: &mut Vec<Event>,
    ) -> Result<bool>;

    fn drain_to_end(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
        events: &mut Vec<Event>,
    ) -> Result<bool> {
        let mut drained = false;
        loop {
            if self.drain_once(ring, pending, events)? {
                drained = true;
            } else {
                break;
            }
        }
        Ok(drained)
    }

    fn feed(&mut self, ring: &mut IoUring, cqe: Cqe, events: &mut Vec<Event>) -> Result<()>;
}
