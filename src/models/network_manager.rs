use rusty_network_manager::{
    AccessPointProxy, DeviceProxy, IP4ConfigProxy, NetworkManagerProxy, WirelessProxy,
};
use zbus::Connection;

use super::singleton;

pub(crate) struct NM {
    connection: Connection,
    nm: NetworkManagerProxy<'static>,
}
singleton!(NM);
impl NM {
    async fn start_if_not_started() {
        if NM::is_set() {
            return;
        }
        let connection = Connection::system()
            .await
            .expect("Could not get a connection.");

        let nm = NetworkManagerProxy::new(&connection)
            .await
            .expect("Could not get NetworkManager");

        Self::set(Self { connection, nm });
    }
}

#[derive(Debug)]
enum WiFiStatusError {
    NoWlan0Iface,
    NoDevice,
    NoActiveAccessPointPath,
    NoActiveAccessPoint,
    AccessPointIsNotAssociated,
    NoSSID,
    NoStrength,
}

#[derive(Debug)]
pub(crate) struct WiFiStatus {
    pub(crate) ssid: String,
    pub(crate) strength: u8,
}
impl WiFiStatus {
    pub(crate) fn spawn<F>(on_change: F)
    where
        F: Fn(Option<WiFiStatus>) + 'static,
    {
        gtk4::glib::spawn_future_local(async move {
            NM::start_if_not_started().await;

            loop {
                match Self::get_state("wlo1").await {
                    Ok(state) => on_change(Some(state)),
                    Err(err) => {
                        on_change(None);
                        eprintln!("WiFiStatus error: {err:?}")
                    }
                }

                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }

    async fn get_state(iface: &str) -> Result<WiFiStatus, WiFiStatusError> {
        let NM { nm, connection } = NM::get();

        let device_path = nm
            .get_device_by_ip_iface(iface)
            .await
            .map_err(|_| WiFiStatusError::NoWlan0Iface)?;
        let device = WirelessProxy::new_from_path(device_path, connection)
            .await
            .map_err(|_| WiFiStatusError::NoDevice)?;

        let access_point_path = device
            .active_access_point()
            .await
            .map_err(|_| WiFiStatusError::NoActiveAccessPointPath)?;

        if access_point_path.as_ref() == "/" {
            return Err(WiFiStatusError::AccessPointIsNotAssociated);
        }

        let access_point = AccessPointProxy::new_from_path(access_point_path, connection)
            .await
            .map_err(|_| WiFiStatusError::NoActiveAccessPoint)?;

        let ssid = String::from_utf8(
            access_point
                .ssid()
                .await
                .map_err(|_| WiFiStatusError::NoSSID)?,
        )
        .unwrap();

        let strength = access_point
            .strength()
            .await
            .map_err(|_| WiFiStatusError::NoStrength)?;

        Ok(WiFiStatus { ssid, strength })
    }
}

#[derive(Debug)]
pub(crate) struct Iface {
    pub(crate) name: String,
    pub(crate) ip: String,
}

pub(crate) async fn all_networks() -> Vec<Iface> {
    NM::start_if_not_started().await;
    let NM { nm, connection } = NM::get();

    let mut ifaces = vec![];
    let device_paths = nm.get_all_devices().await.unwrap();
    for device_path in device_paths {
        let device = DeviceProxy::new_from_path(device_path, connection)
            .await
            .expect("Error");
        let ip4config_path = device.ip4_config().await.unwrap();
        let ip4config = IP4ConfigProxy::new_from_path(ip4config_path, connection).await;

        if let Ok(config) = ip4config {
            if let Ok(address_data) = config.address_data().await {
                if let Some(address) = address_data.first().and_then(|addr| addr.get("address")) {
                    let name = device.interface().await.unwrap();
                    let ip = address.downcast_ref::<String>().unwrap();
                    ifaces.push(Iface { name, ip });
                }
            }
        }
    }

    ifaces
}
