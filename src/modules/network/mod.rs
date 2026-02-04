use crate::{
    Event,
    dbus::{DBus, Message},
    liburing::IoUring,
};
use active_access_point::{ActiveAccessPoint, ActiveAccessPointEvent};
use anyhow::Result;
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
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            wireless_connection: WirelessConnection::new(),
            primary_device: PrimaryDevice::new(),
            active_access_point: ActiveAccessPoint::new(),
            tx_rx: TxRx::new(),
            speed: Speed::new(),
            ssid_and_strength: SsidAndStrength::new(),
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.wireless_connection.init(dbus, ring)
    }

    fn on_wireless_connection_event(
        &mut self,
        dbus: &mut DBus,
        e: WirelessConnectionEvent,
        ring: &mut IoUring,
    ) -> Result<()> {
        match e {
            WirelessConnectionEvent::Connected(path) => {
                self.primary_device.init(path, dbus, ring)?;
            }
            WirelessConnectionEvent::Disconnected => {
                self.primary_device.reset(dbus, ring)?;
            }
        }
        Ok(())
    }

    fn on_primary_device_event(
        &mut self,
        dbus: &mut DBus,
        e: PrimaryDeviceEvent,
        ring: &mut IoUring,
    ) -> Result<()> {
        match e {
            PrimaryDeviceEvent::Connected(path) => {
                self.active_access_point.init(dbus, &path, ring)?;
                self.speed.reset();
                self.tx_rx.init(dbus, &path, ring)?;
            }
            PrimaryDeviceEvent::Disconnected => {
                self.active_access_point.reset(dbus, ring)?;
                self.speed.reset();
                self.tx_rx.reset(dbus, ring)?;
            }
        }

        Ok(())
    }

    fn on_active_access_point_event(
        &mut self,
        dbus: &mut DBus,
        e: ActiveAccessPointEvent,
        ring: &mut IoUring,
    ) -> Result<()> {
        match e {
            ActiveAccessPointEvent::Connected(path) => {
                self.ssid_and_strength.init(dbus, &path, ring)?;
            }
            ActiveAccessPointEvent::Disconnected => {
                self.ssid_and_strength.reset(dbus, ring)?;
            }
        }

        Ok(())
    }

    fn on_tx_rx_event(&mut self, e: TxRxEvent, events: &mut Vec<Event>) {
        if let Some(tx) = e.tx {
            let event = self.speed.update_tx(tx);
            events.push(event);
        }

        if let Some(rx) = e.rx {
            let event = self.speed.update_rx(rx);
            events.push(event);
        }
    }

    fn on_ssid_and_strength_event(&mut self, e: SsidAndStrengthEvent, events: &mut Vec<Event>) {
        if let Some(ssid) = e.ssid {
            let event = Event::NetworkSsid { ssid: ssid.into() };
            events.push(event)
        }

        if let Some(strength) = e.strength {
            let event = Event::NetworkStrength { strength };
            events.push(event)
        }
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        events: &mut Vec<Event>,
        ring: &mut IoUring,
    ) -> Result<()> {
        if let Some(e) = self.wireless_connection.on_message(dbus, message, ring)? {
            self.on_wireless_connection_event(dbus, e, ring)?;
            return Ok(());
        }

        if let Some(e) = self.primary_device.on_message(message) {
            self.on_primary_device_event(dbus, e, ring)?;
            return Ok(());
        }

        if let Some(e) = self.active_access_point.on_message(message) {
            self.on_active_access_point_event(dbus, e, ring)?;
            return Ok(());
        }

        if let Some(e) = self.tx_rx.on_message(message) {
            self.on_tx_rx_event(e, events);
            return Ok(());
        }

        if let Some(e) = self.ssid_and_strength.on_message(message) {
            self.on_ssid_and_strength_event(e, events);
        }

        Ok(())
    }
}
