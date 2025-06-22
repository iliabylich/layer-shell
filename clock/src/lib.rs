#![expect(clippy::new_without_default)]

mod clock;
mod event;
mod task;

pub use clock::Clock;
pub use event::Event;
