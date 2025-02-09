use crate::{dbus::nm::NetworkManager, event::WifiStatus, ffi::COption, Event};
use anyhow::{Context as _, Result};
use dbus::blocking::Connection;
use std::sync::mpsc::Sender;

pub(crate) fn reset(conn: &Connection, tx: &Sender<Event>) -> Result<()> {
    let wifi_status = match get_status(conn) {
        Ok((ssid, strength)) => COption::Some(WifiStatus {
            ssid: ssid.into(),
            strength,
        }),
        Err(err) => {
            log::warn!("WiFiStatus error: {:?}", err);
            COption::None
        }
    };

    let event = Event::WifiStatus { wifi_status };
    tx.send(event).context("failed to send WiFiStatus event")?;
    Ok(())
}
fn get_status(conn: &Connection) -> Result<(String, u8)> {
    let device = NetworkManager::primary_wireless_device(conn)?;
    let access_point = device.active_access_point(conn)?;
    let ssid = access_point.ssid(conn)?;
    let strength = access_point.strength(conn)?;

    Ok((ssid, strength))
}
