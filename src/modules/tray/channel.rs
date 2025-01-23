use crate::lock_channel::LockChannel;
use std::sync::LazyLock;

#[derive(Debug)]
pub(crate) enum Command {
    ServiceAdded { service: String, path: String },
    ServiceRemoved { service: String },
    ServiceUpdated { service: String },

    TriggerItem { uuid: String },
}

pub(crate) static CHANNEL: LazyLock<LockChannel<Command>> = LazyLock::new(LockChannel::new);
