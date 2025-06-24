#[must_use]
#[derive(Debug)]
pub enum Event {
    WifiStatus { wifi_status: Option<WifiStatus> },
    UploadSpeed { speed: String },
    DownloadSpeed { speed: String },
    NetworkList { list: Vec<NetworkData> },
}

#[derive(Debug)]
pub struct NetworkData {
    pub iface: String,
    pub address: String,
}

#[derive(Debug)]
pub struct WifiStatus {
    pub ssid: String,
    pub strength: u8,
}
