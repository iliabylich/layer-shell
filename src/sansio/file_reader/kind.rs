use anyhow::{Result, ensure};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[expect(clippy::upper_case_acronyms)]
pub(crate) enum FileReaderKind {
    CPU,
    Memory,

    MAX,
}
const COUNT: usize = FileReaderKind::MAX as usize;

const BUF_SIZE: usize = 1_024;
static mut BUFFERS: [[u8; BUF_SIZE]; COUNT] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

impl FileReaderKind {
    pub(crate) fn buffer(self) -> Result<&'static mut [u8; BUF_SIZE]> {
        ensure!(
            (self as usize) < (Self::MAX as usize),
            "unknown file kind: {}",
            self as usize
        );

        unsafe { Ok(&mut BUFFERS[self as usize]) }
    }
}
