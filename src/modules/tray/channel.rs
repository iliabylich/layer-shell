use crate::lock_channel::LockChannel;
use std::sync::LazyLock;

#[derive(Debug)]
pub(crate) enum TrayCommand {
    Added { service: String, path: String },
    Removed { service: String },
    Updated { service: String },
}

pub(crate) static CHANNEL: LazyLock<LockChannel<TrayCommand>> = LazyLock::new(LockChannel::new);
