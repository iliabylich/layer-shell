use crate::NetworkData;
use ffi::{CArray, CString};

#[derive(Debug)]
pub enum NetworkEvent {
    Ssid(NetworkSsidEvent),
    Strength(NetworkStrengthEvent),
    UploadSpeed(UploadSpeedEvent),
    DownloadSpeed(DownloadSpeedEvent),
    NetworkList(NetworkListEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct UploadSpeedEvent {
    pub speed: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct DownloadSpeedEvent {
    pub speed: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkListEvent {
    pub list: CArray<NetworkData>,
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkSsidEvent {
    pub ssid: CString,
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkStrengthEvent {
    pub strength: u8,
}
