use crate::{Event, event_queue::EventQueue, utils::StringRef};
use active_access_point::{ActiveAccessPoint, ActiveAccessPointEvent};
use dbus::IncomingMessage;
use primary_device::{PrimaryDevice, PrimaryDeviceEvent};
use speed::Speed;
use ssid_and_strength::{SsidAndStrength, SsidAndStrengthEvent};
use tx_rx::{TxRx, TxRxEvent};
use wireless_connection::{WirelessConnection, WirelessConnectionEvent};

mod active_access_point;
mod active_connection_type;
mod primary_connection;
mod primary_device;
mod speed;
mod ssid_and_strength;
mod tx_rx;
mod wireless_connection;

pub(crate) struct Network {
    wireless_connection: WirelessConnection,
    primary_device: PrimaryDevice,
    active_access_point: ActiveAccessPoint,
    tx_rx: TxRx,
    speed: Speed,
    ssid_and_strength: SsidAndStrength,

    last_ssid: Option<StringRef>,
    last_strength: Option<u8>,
}

impl Network {
    pub(crate) fn new() -> Self {
        Self {
            wireless_connection: WirelessConnection::new(),
            primary_device: PrimaryDevice::new(),
            active_access_point: ActiveAccessPoint::new(),
            tx_rx: TxRx::new(),
            speed: Speed::new(),
            ssid_and_strength: SsidAndStrength::new(),
            last_ssid: None,
            last_strength: None,
        }
    }

    pub(crate) fn init(&mut self) {
        self.wireless_connection.start();
    }

    fn on_wireless_connection_event(&mut self, e: WirelessConnectionEvent) {
        match e {
            WirelessConnectionEvent::Connected(path) => {
                self.primary_device.start(path);
            }
            WirelessConnectionEvent::Disconnected => {
                self.primary_device.stop();
            }
        }
    }

    fn on_primary_device_event(&mut self, e: PrimaryDeviceEvent) {
        match e {
            PrimaryDeviceEvent::Connected(path) => {
                self.active_access_point.start(path.clone());
                self.speed.reset();
                self.tx_rx.start(path);
            }
            PrimaryDeviceEvent::Disconnected => {
                self.active_access_point.stop();
                self.speed.reset();
                self.tx_rx.stop();
            }
        }
    }

    fn on_active_access_point_event(&mut self, e: ActiveAccessPointEvent) {
        match e {
            ActiveAccessPointEvent::Connected(path) => {
                self.ssid_and_strength.start(path);
            }
            ActiveAccessPointEvent::Disconnected => {
                self.ssid_and_strength.stop();
            }
        }
    }

    fn on_tx_rx_event(&mut self, e: TxRxEvent) {
        if let Some(tx) = e.tx {
            let event = self.speed.update_tx(tx);
            EventQueue::push_back(event);
        }

        if let Some(rx) = e.rx {
            let event = self.speed.update_rx(rx);
            EventQueue::push_back(event);
        }
    }

    #[expect(clippy::useless_let_if_seq)]
    fn on_ssid_and_strength_event(&mut self, e: SsidAndStrengthEvent) {
        let mut got_diff = false;

        if let Some(ssid) = e.ssid
            && self.last_ssid != Some(ssid.clone())
        {
            self.last_ssid = Some(ssid);
            got_diff = true;
        }

        if let Some(strength) = e.strength
            && self.last_strength != Some(strength)
        {
            self.last_strength = Some(strength);
            got_diff = true;
        }

        if got_diff
            && let Some(ssid) = self.last_ssid.clone()
            && let Some(strength) = self.last_strength
        {
            EventQueue::push_back(Event::NetworkSsidAndStrength { ssid, strength });
        }
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) {
        if let Some(e) = self.wireless_connection.handle(message) {
            self.on_wireless_connection_event(e);
            return;
        }

        if let Some(e) = self.primary_device.handle(message) {
            self.on_primary_device_event(e);
            return;
        }

        if let Some(e) = self.active_access_point.handle(message) {
            self.on_active_access_point_event(e);
            return;
        }

        if let Some(e) = self.tx_rx.handle(message) {
            self.on_tx_rx_event(e);
            return;
        }

        if let Some(e) = self.ssid_and_strength.handle(message) {
            self.on_ssid_and_strength_event(e);
        }
    }
}
