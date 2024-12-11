use crate::{event::WiFiStatus, Event};
use anyhow::Result;
use dbus::nonblock::SyncConnection;
use layer_shell_dbus::nm::NetworkManager;

pub(crate) async fn get(conn: &SyncConnection) -> Event {
    let state = match get_status(conn, "wlo1").await {
        Ok(state) => Some(state),
        Err(err) => {
            log::error!("WiFiStatus error: {:?}", err);
            None
        }
    };

    Event::WiFiStatus(state)
}

async fn get_status(conn: &SyncConnection, iface: &str) -> Result<WiFiStatus> {
    let device = NetworkManager::get_device_by_ip_iface(conn, iface).await?;
    let access_point = device.active_access_point(conn).await?;
    let ssid = access_point.ssid(conn).await?;
    let strength = access_point.strength(conn).await?;

    Ok(WiFiStatus { ssid, strength })
}
