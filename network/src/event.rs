use ffi::{CArray, COption, CString};

#[derive(Debug)]
pub enum NetworkEvent {
    WifiStatus(WifiStatusEvent),
    UploadSpeed(UploadSpeedEvent),
    DownloadSpeed(DownloadSpeedEvent),
    NetworkList(NetworkListEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct WifiStatusEvent {
    pub wifi_status: COption<WifiStatus>,
}

#[derive(Debug)]
#[repr(C)]
pub struct UploadSpeedEvent {
    pub speed: String,
}

#[derive(Debug)]
#[repr(C)]
pub struct DownloadSpeedEvent {
    pub speed: String,
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkListEvent {
    pub list: CArray<NetworkData>,
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkData {
    pub iface: CString,
    pub address: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct WifiStatus {
    pub ssid: CString,
    pub strength: u8,
}
