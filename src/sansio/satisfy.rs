use crate::utils::assert_or_exit;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub(crate) enum Satisfy {
    Socket,
    Connect,
    Write,
    Read,
    Close,
    OpenAt,
    Crash,
}
const MAX: Satisfy = Satisfy::Crash;

impl From<Satisfy> for u8 {
    fn from(value: Satisfy) -> Self {
        value as u8
    }
}

impl From<u8> for Satisfy {
    fn from(value: u8) -> Self {
        assert_or_exit!(
            value <= MAX as u8,
            "received malformed Satisfy from io_uring: {value}"
        );
        unsafe { core::mem::transmute::<u8, Self>(value) }
    }
}
