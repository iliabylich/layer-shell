use crate::{event::WiFiStatus, Event};
use anyhow::Result;
use dbus::nonblock::SyncConnection;
use layer_shell_dbus::nm::NetworkManager;
use std::sync::{mpsc::Sender, Arc};

pub(crate) async fn spawn(tx: Sender<Event>, conn: Arc<SyncConnection>) {
    loop {
        if let Err(err) = tick(&tx, conn.as_ref()).await {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn tick(tx: &Sender<Event>, conn: &SyncConnection) -> Result<()> {
    let state = get_status(conn, "wlo1")
        .await
        .inspect_err(|err| log::error!("WiFiStatus error: {:?}", err))
        .ok();

    tx.send(Event::WiFiStatus(state))?;

    Ok(())
}

async fn get_status(conn: &SyncConnection, iface: &str) -> Result<WiFiStatus> {
    let device = NetworkManager::get_device_by_ip_iface(conn, iface).await?;
    let access_point = device.active_access_point(conn).await?;
    let ssid = access_point.ssid(conn).await?;
    let strength = access_point.strength(conn).await?;

    Ok(WiFiStatus { ssid, strength })
}
