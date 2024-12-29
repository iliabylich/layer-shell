use crate::{dbus::nm::NetworkManager, Event};
use anyhow::Result;
use dbus::nonblock::SyncConnection;

pub(crate) async fn get(conn: &SyncConnection) -> Event {
    let (ssid, strength) = match get_status(conn, "wlo1").await {
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

async fn get_status(conn: &SyncConnection, iface: &str) -> Result<(String, u8)> {
    let device = NetworkManager::get_device_by_ip_iface(conn, iface).await?;
    let access_point = device.active_access_point(conn).await?;
    let ssid = access_point.ssid(conn).await?;
    let strength = access_point.strength(conn).await?;

    Ok((ssid, strength))
}
