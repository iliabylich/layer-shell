use crate::{Event, event_queue::EventQueue, utils::StringRef};
use active_access_point::{ActiveAccessPoint, ActiveAccessPointEvent};
use anyhow::Result;
use mini_sansio_dbus::IncomingMessage;
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
        }
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        self.wireless_connection.init()?;
        Ok(())
    }

    fn on_wireless_connection_event(&mut self, e: WirelessConnectionEvent) -> Result<()> {
        match e {
            WirelessConnectionEvent::Connected(path) => {
                self.primary_device.init(path)?;
            }
            WirelessConnectionEvent::Disconnected => {
                self.primary_device.reset();
            }
        }
        Ok(())
    }

    fn on_primary_device_event(&mut self, e: PrimaryDeviceEvent) -> Result<()> {
        match e {
            PrimaryDeviceEvent::Connected(path) => {
                self.active_access_point.init(path.clone())?;
                self.speed.reset();
                self.tx_rx.init(path)?;
            }
            PrimaryDeviceEvent::Disconnected => {
                self.active_access_point.reset();
                self.speed.reset();
                self.tx_rx.reset();
            }
        }
        Ok(())
    }

    fn on_active_access_point_event(&mut self, e: ActiveAccessPointEvent) -> Result<()> {
        match e {
            ActiveAccessPointEvent::Connected(path) => {
                self.ssid_and_strength.init(path)?;
            }
            ActiveAccessPointEvent::Disconnected => {
                self.ssid_and_strength.reset();
            }
        }
        Ok(())
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

    fn on_ssid_and_strength_event(&mut self, e: SsidAndStrengthEvent) -> Result<()> {
        if let Some(ssid) = e.ssid {
            let event = Event::NetworkSsid {
                ssid: StringRef::new(ssid.as_str())?,
            };
            EventQueue::push_back(event)
        }

        if let Some(strength) = e.strength {
            let event = Event::NetworkStrength { strength };
            EventQueue::push_back(event)
        }

        Ok(())
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Result<()> {
        if let Some(e) = self.wireless_connection.on_message(message)? {
            self.on_wireless_connection_event(e)?;
            return Ok(());
        }

        if let Some(e) = self.primary_device.on_message(message) {
            self.on_primary_device_event(e)?;
            return Ok(());
        }

        if let Some(e) = self.active_access_point.on_message(message) {
            self.on_active_access_point_event(e)?;
            return Ok(());
        }

        if let Some(e) = self.tx_rx.on_message(message) {
            self.on_tx_rx_event(e);
            return Ok(());
        }

        if let Some(e) = self.ssid_and_strength.on_message(message) {
            return self.on_ssid_and_strength_event(e);
        }

        Ok(())
    }
}
