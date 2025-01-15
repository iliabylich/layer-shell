use crate::{dbus::nm::NetworkManager, Event};
use anyhow::{bail, Result};
use dbus::blocking::Connection;

pub(crate) fn get(conn: &Connection) -> Event {
    let (ssid, strength) = match get_status(conn) {
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
fn get_status(conn: &Connection) -> Result<(String, u8)> {
    let primary_connection = NetworkManager::primary_connection(conn)?;
    if !primary_connection.type_(conn)?.contains("wireless") {
        return Ok((String::new(), 0));
    }

    let devices = primary_connection.devices(conn)?;
    if devices.len() != 1 {
        bail!("NM returned multiple devices for active connection");
    }
    let device = devices.into_iter().next().unwrap();

    let access_point = device.active_access_point(conn)?;
    let ssid = access_point.ssid(conn)?;
    let strength = access_point.strength(conn)?;

    Ok((ssid, strength))
}
