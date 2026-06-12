use crate::sansio::Op;
use anyhow::Result;
use std::os::fd::BorrowedFd;

#[derive(Debug)]
pub(crate) enum Satisfy {
    Socket(Result<BorrowedFd<'static>>),
    Connect(Result<()>),
    Write(Result<usize>),
    Read(Result<usize>),
    Close(Result<()>),
    OpenAt(Result<BorrowedFd<'static>>),
}

impl Satisfy {
    pub(crate) fn new(op: Op, res: i32) -> Self {
        match op {
            Op::Socket => {
                let fd = if res >= 0 {
                    Ok(unsafe { BorrowedFd::borrow_raw(res) })
                } else {
                    Err(anyhow::anyhow!("Op::Socket returned error: {res}"))
                };
                Self::Socket(fd)
            }
            Op::Connect => {
                let res = if res >= 0 {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Op::Connect returned error: {res}"))
                };
                Self::Connect(res)
            }
            Op::Write => {
                let len = usize::try_from(res)
                    .map_err(|_| anyhow::anyhow!("Op::Write returned error: {res}"));
                Self::Write(len)
            }
            Op::Read => {
                let len = usize::try_from(res)
                    .map_err(|_| anyhow::anyhow!("Op::Read returned error: {res}"));
                Self::Read(len)
            }
            Op::Close => {
                let res = if res >= 0 {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Op::Close returned error: {res}"))
                };
                Self::Close(res)
            }
            Op::OpenAt => {
                let fd = if res >= 0 {
                    Ok(unsafe { BorrowedFd::borrow_raw(res) })
                } else {
                    Err(anyhow::anyhow!("Op::OpenAt returned error: {res}"))
                };
                Self::OpenAt(fd)
            }
        }
    }
}
