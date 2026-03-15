use crate::{Event, dbus::decoder::IncomingMessage, event_queue::EventQueue, sansio::DBusQueue};
use active_access_point::{ActiveAccessPoint, ActiveAccessPointEvent};
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
    events: EventQueue,
}

impl Network {
    pub(crate) fn new(events: EventQueue, queue: DBusQueue) -> Self {
        Self {
            wireless_connection: WirelessConnection::new(queue.clone()),
            primary_device: PrimaryDevice::new(queue.clone()),
            active_access_point: ActiveAccessPoint::new(queue.clone()),
            tx_rx: TxRx::new(queue.clone()),
            speed: Speed::new(),
            ssid_and_strength: SsidAndStrength::new(queue.clone()),
            events,
        }
    }

    pub(crate) fn init(&mut self) {
        self.wireless_connection.init()
    }

    fn on_wireless_connection_event(&mut self, e: WirelessConnectionEvent) {
        match e {
            WirelessConnectionEvent::Connected(path) => {
                self.primary_device.init(path);
            }
            WirelessConnectionEvent::Disconnected => {
                self.primary_device.reset();
            }
        }
    }

    fn on_primary_device_event(&mut self, e: PrimaryDeviceEvent) {
        match e {
            PrimaryDeviceEvent::Connected(path) => {
                self.active_access_point.init(&path);
                self.speed.reset();
                self.tx_rx.init(&path);
            }
            PrimaryDeviceEvent::Disconnected => {
                self.active_access_point.reset();
                self.speed.reset();
                self.tx_rx.reset();
            }
        }
    }

    fn on_active_access_point_event(&mut self, e: ActiveAccessPointEvent) {
        match e {
            ActiveAccessPointEvent::Connected(path) => {
                self.ssid_and_strength.init(&path);
            }
            ActiveAccessPointEvent::Disconnected => {
                self.ssid_and_strength.reset();
            }
        }
    }

    fn on_tx_rx_event(&mut self, e: TxRxEvent) {
        if let Some(tx) = e.tx {
            let event = self.speed.update_tx(tx);
            self.events.push_back(event);
        }

        if let Some(rx) = e.rx {
            let event = self.speed.update_rx(rx);
            self.events.push_back(event);
        }
    }

    fn on_ssid_and_strength_event(&mut self, e: SsidAndStrengthEvent) {
        if let Some(ssid) = e.ssid {
            let event = Event::NetworkSsid { ssid: ssid.into() };
            self.events.push_back(event)
        }

        if let Some(strength) = e.strength {
            let event = Event::NetworkStrength { strength };
            self.events.push_back(event)
        }
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
        if let Some(e) = self.wireless_connection.on_message(message) {
            self.on_wireless_connection_event(e);
            return;
        }

        if let Some(e) = self.primary_device.on_message(message) {
            self.on_primary_device_event(e);
            return;
        }

        if let Some(e) = self.active_access_point.on_message(message) {
            self.on_active_access_point_event(e);
            return;
        }

        if let Some(e) = self.tx_rx.on_message(message) {
            self.on_tx_rx_event(e);
            return;
        }

        if let Some(e) = self.ssid_and_strength.on_message(message) {
            self.on_ssid_and_strength_event(e);
        }
    }
}
