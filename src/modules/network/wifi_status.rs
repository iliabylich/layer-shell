use crate::{dbus::nm::NetworkManager, event::WifiStatus, ffi::COption, Event};
use anyhow::Result;
use dbus::blocking::Connection;

pub(crate) fn get(conn: &Connection) -> Event {
    let wifi_status = match get_status(conn) {
        Ok((ssid, strength)) => COption::Some(WifiStatus {
            ssid: ssid.into(),
            strength,
        }),
        Err(err) => {
            log::error!("WiFiStatus error: {:?}", err);
            COption::None
        }
    };

    Event::WifiStatus { wifi_status }
}
fn get_status(conn: &Connection) -> Result<(String, u8)> {
    let device = NetworkManager::primary_wireless_device(conn)?;
    let access_point = device.active_access_point(conn)?;
    let ssid = access_point.ssid(conn)?;
    let strength = access_point.strength(conn)?;

    Ok((ssid, strength))
}
