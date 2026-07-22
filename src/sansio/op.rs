use crate::utils::log_err_and_exit;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Op {
    Socket,
    Connect,
    Write,
    Read,
    Accept,
}

impl Op {
    pub(crate) fn new(op: u8) -> Self {
        if op == Self::Socket as u8 {
            Self::Socket
        } else if op == Self::Connect as u8 {
            Self::Connect
        } else if op == Self::Write as u8 {
            Self::Write
        } else if op == Self::Read as u8 {
            Self::Read
        } else if op == Self::Accept as u8 {
            Self::Accept
        } else {
            log_err_and_exit!("unknown op {op}")
        }
    }
}
