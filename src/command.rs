#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum Command {
    Lock,
    Reboot,
    Shutdown,
    Logout,

    SpawnWiFiEditor,
    SpawnBluetoothEditor,
    SpawnSystemMonitor,
    ChangeWallpaper,

    TriggerTray { service: u32, id: i32 },
}
