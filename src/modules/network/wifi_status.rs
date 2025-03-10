use crate::{Event, dbus::nm::Device, event::WifiStatus, ffi::COption};
use anyhow::Result;
use dbus::blocking::Connection;

pub(crate) fn load(device: &Device, conn: &Connection) -> Event {
    let wifi_status = match get_wifi_status(device, conn) {
        Ok((ssid, strength)) => COption::Some(WifiStatus {
            ssid: ssid.into(),
            strength,
        }),
        Err(err) => {
            log::warn!("WiFiStatus error: {:?}", err);
            COption::None
        }
    };

    Event::WifiStatus { wifi_status }
}

fn get_wifi_status(device: &Device, conn: &Connection) -> Result<(String, u8)> {
    let access_point = device.active_access_point(conn)?;
    let ssid = access_point.ssid(conn)?;
    let strength = access_point.strength(conn)?;

    Ok((ssid, strength))
}
