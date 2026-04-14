use crate::utils::assert_or_exit;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum DBusConnectionKind {
    System,
    Session,
}

const COUNT: usize = DBusConnectionKind::Session as usize + 1;

static mut BUFFERS: [[u8; DBusConnectionKind::READ_BUF_SIZE]; COUNT] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

impl DBusConnectionKind {
    pub(crate) const READ_BUF_SIZE: usize = 500_000;

    pub(crate) fn read_buffer(self) -> &'static mut [u8; Self::READ_BUF_SIZE] {
        assert_or_exit!(
            (self as usize) < COUNT,
            "unknown file kind: {}",
            self as usize
        );

        unsafe { &mut BUFFERS[self as usize] }
    }
}
