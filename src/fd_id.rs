use crate::fatal;

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub(crate) enum FdId {
    Timer,
    HyprlandSocket,
    Command,
    ControlDBus,
    PipewireDBus,
    LauncherGlobalDirInotify,
    LauncherUserDirInotify,
    NetworkDBus,
    TrayDBus,
    Weather,

    Unknown,
}
impl FdId {
    pub(crate) const fn token(self) -> mio::Token {
        mio::Token(self as usize)
    }
}
impl From<usize> for FdId {
    fn from(value: usize) -> Self {
        if value >= Self::Unknown as usize {
            fatal!("invalid fd id {value}");
        }
        unsafe { std::mem::transmute::<usize, Self>(value) }
    }
}
impl From<mio::Token> for FdId {
    fn from(token: mio::Token) -> Self {
        FdId::from(usize::from(token))
    }
}
