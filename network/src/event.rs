#[derive(Debug)]
pub enum Event {
    WiFiStatus(Option<WiFiStatus>),
    NetworkList(NetworkList),
}

#[derive(Debug)]
pub struct WiFiStatus {
    pub ssid: String,
    pub strength: u8,
}

#[derive(Debug)]
pub struct NetworkList {
    pub list: Vec<Network>,
}

#[derive(Debug)]
pub struct Network {
    pub iface: String,
    pub address: String,
}
