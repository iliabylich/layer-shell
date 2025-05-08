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
}
impl FdId {
    pub(crate) const fn token(self) -> mio::Token {
        mio::Token(self as usize)
    }
}
