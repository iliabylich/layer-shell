#[derive(Debug, Clone)]
#[must_use]
pub(crate) enum Command {
    FinishIoThread,

    HyprlandGoToWorkspace { workspace: usize },

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnWiFiEditor,
    SpawnBluetoothEditor,
    SpawnSystemMonitor,
    ChangeTheme,
}
