use crate::{dbus::nm::NetworkManager, Event};
use anyhow::Result;
use dbus::blocking::Connection;

pub(crate) fn get(conn: &Connection) -> Event {
    let (ssid, strength) = match get_status(conn, "wlo1") {
        Ok(state) => state,
        Err(err) => {
            log::error!("WiFiStatus error: {:?}", err);
            (String::new(), 0)
        }
    };

    Event::WiFiStatus {
        ssid: ssid.into(),
        strength,
    }
}
fn get_status(conn: &Connection, iface: &str) -> Result<(String, u8)> {
    let device = NetworkManager::get_device_by_ip_iface(conn, iface)?;
    let access_point = device.active_access_point(conn)?;
    let ssid = access_point.ssid(conn)?;
    let strength = access_point.strength(conn)?;

    Ok((ssid, strength))
}
