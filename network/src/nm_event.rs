use zbus::zvariant::OwnedObjectPath;

#[derive(Debug)]
pub(crate) enum NetworkManagerEvent {
    PrimaryConnection(OwnedObjectPath),
    PrimaryDevices(Vec<OwnedObjectPath>),
    AccessPoint(OwnedObjectPath),
    Ssid(Vec<u8>),
    Strength(u8),
    Devices(Vec<OwnedObjectPath>),
    DeviceTxBytes(u64),
    DeviceRxBytes(u64),
}
