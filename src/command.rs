#[derive(Debug, Clone)]
#[must_use]
pub(crate) enum Command {
    FinishIoThread,

    HyprlandGoToWorkspace { idx: usize },

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnNetworkEditor,
    SpawnSystemMonitor,
    ChangeTheme,
}
