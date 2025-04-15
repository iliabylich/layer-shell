#[derive(Debug, Clone)]
#[must_use]
pub(crate) enum Command {
    HyprlandGoToWorkspace { idx: usize },

    LauncherReset,
    LauncherGoUp,
    LauncherGoDown,
    LauncherSetSearch { search: String },
    LauncherExecSelected,

    Lock,
    Reboot,
    Shutdown,
    Logout,

    TriggerTray { uuid: String },

    SpawnNetworkEditor,
    SpawnSystemMonitor,
    ChangeTheme,
}
