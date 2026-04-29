use crate::utils::StringRef;

#[derive(Debug)]
#[must_use]
pub(crate) enum Command {
    Lock,
    Reboot,
    Shutdown,
    Logout,

    SpawnWiFiEditor,
    SpawnBluetoothEditor,
    SpawnSystemMonitor,
    ChangeTheme,

    TriggerTray { uuid: StringRef },
}
