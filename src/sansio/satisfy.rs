use crate::{error::IoError, sansio::Op};
use rustix::{fd::BorrowedFd, io::Errno};

#[derive(Debug, Clone, Copy)]
pub enum Satisfy {
    Socket(Result<BorrowedFd<'static>, IoError>),
    Connect(Result<(), IoError>),
    Write(Result<usize, IoError>),
    Read(Result<usize, IoError>),
    Accept(Result<BorrowedFd<'static>, IoError>),
}

impl Satisfy {
    pub(crate) fn new(op: Op, res: i32) -> Self {
        match op {
            Op::Socket => Self::Socket(if res >= 0 {
                Ok(unsafe { BorrowedFd::borrow_raw(res) })
            } else {
                Err(IoError::FailedTo {
                    op: "socket",
                    errno: Errno::from_raw_os_error(-res),
                })
            }),
            Op::Connect => Self::Connect(if res >= 0 {
                Ok(())
            } else {
                Err(IoError::FailedTo {
                    op: "connect",
                    errno: Errno::from_raw_os_error(-res),
                })
            }),
            Op::Write => Self::Write(usize::try_from(res).map_err(|_| IoError::FailedTo {
                op: "write",
                errno: Errno::from_raw_os_error(-res),
            })),
            Op::Read => Self::Read(usize::try_from(res).map_err(|_| IoError::FailedTo {
                op: "read",
                errno: Errno::from_raw_os_error(-res),
            })),
            Op::Accept => Self::Accept(if res >= 0 {
                Ok(unsafe { BorrowedFd::borrow_raw(res) })
            } else {
                Err(IoError::FailedTo {
                    op: "accept",
                    errno: Errno::from_raw_os_error(-res),
                })
            }),
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Socket(_) => "Socket",
            Self::Connect(_) => "Connect",
            Self::Write(_) => "Write",
            Self::Read(_) => "Read",
            Self::Accept(_) => "Accept",
        }
    }
}
