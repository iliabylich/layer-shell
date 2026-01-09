use crate::dbus::types::{Flags, MessageType};

#[derive(Default, Eq, Clone, Copy)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: Flags,
    pub(crate) body_len: usize,
    pub(crate) serial: u32,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?})", self.message_type, self.flags)
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Self) -> bool {
        self.message_type == other.message_type && self.flags == other.flags
    }
}
