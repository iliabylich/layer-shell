use rusty_network_manager::{AccessPointProxy, WirelessProxy};

use crate::{models::NM, utils::Singleton};

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
