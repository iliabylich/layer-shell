use crate::{sansio::Op, user_data::ModuleId};
use rustix::{fd::BorrowedFd, io::Errno};

#[derive(Debug, Clone, Copy)]
pub enum Satisfy {
    Socket(Result<BorrowedFd<'static>, ()>),
    Connect(Result<(), ()>),
    Write(Result<usize, ()>),
    Read(Result<usize, ()>),
    Accept(Result<BorrowedFd<'static>, ()>),
}

impl Satisfy {
    pub(crate) fn new(op: Op, res: i32, scope: ModuleId) -> Self {
        log::trace!("Creating Satisfy");

        let log_error = |op: &str| {
            log::error!(
                "failed to {op} at {scope:?}: {:?}",
                Errno::from_raw_os_error(-res)
            );
        };

        match op {
            Op::Socket => {
                let res = if res >= 0 {
                    Ok(unsafe { BorrowedFd::borrow_raw(res) })
                } else {
                    log_error("socket");
                    Err(())
                };
                Self::Socket(res)
            }

            Op::Connect => {
                let res = if res >= 0 {
                    Ok(())
                } else {
                    log_error("connect");
                    Err(())
                };
                Self::Connect(res)
            }

            Op::Write => {
                let res = usize::try_from(res).map_err(|_| {
                    log_error("write");
                });
                Self::Write(res)
            }

            Op::Read => {
                let res = usize::try_from(res).map_err(|_| {
                    log_error("read");
                });
                Self::Read(res)
            }

            Op::Accept => {
                let res = if res >= 0 {
                    Ok(unsafe { BorrowedFd::borrow_raw(res) })
                } else {
                    log_error("accept");
                    Err(())
                };
                Self::Accept(res)
            }
        }
    }
}
