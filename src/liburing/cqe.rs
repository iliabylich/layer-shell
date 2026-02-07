use crate::liburing::io_uring_cqe;

#[derive(Clone, Copy)]
pub(crate) struct Cqe {
    pub(crate) cqe: *mut io_uring_cqe,
}

impl Cqe {
    pub(crate) const fn res(&self) -> i32 {
        unsafe { (*self.cqe).res }
    }

    pub(crate) const fn user_data(&self) -> u64 {
        unsafe { (*self.cqe).user_data }
    }
}
