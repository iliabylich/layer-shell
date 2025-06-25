use crate::{
    NetworkEvent, WifiStatusEvent, event::WifiStatus, nm_event::NetworkManagerEvent,
    stream_map::StreamMap,
};
use anyhow::Result;
use ffi::COption;
use futures::StreamExt;
use zbus::{Connection, zvariant::OwnedObjectPath};

mod dbus {
    use zbus::proxy;

    #[proxy(
        default_service = "org.freedesktop.NetworkManager",
        interface = "org.freedesktop.NetworkManager.AccessPoint",
        assume_defaults = true
    )]
    pub(crate) trait AccessPoint {
        #[zbus(property)]
        fn ssid(&self) -> zbus::Result<Vec<u8>>;

        #[zbus(property)]
        fn strength(&self) -> zbus::Result<u8>;
    }
}

pub(crate) struct AccessPoint {
    path: Option<OwnedObjectPath>,
    ssid: Option<String>,
    strength: Option<u8>,
}

const SSID_CHANGED: &str = "SSID_CHANGED";
const STRENGTH_CHANGED: &str = "STRENGTH_CHANGED";

impl AccessPoint {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            ssid: None,
            strength: None,
        }
    }

    pub(crate) fn clear(&mut self, stream_map: &mut StreamMap) {
        self.path = None;
        self.ssid = None;
        self.strength = None;

        stream_map.remove(SSID_CHANGED);
        stream_map.remove(STRENGTH_CHANGED);
    }

    pub(crate) async fn changed(
        &mut self,
        path: OwnedObjectPath,
        conn: &Connection,
        stream_map: &mut StreamMap,
    ) -> Result<()> {
        if self.path.as_ref().is_some_and(|v| v == &path) {
            return Ok(());
        }

        self.clear(stream_map);

        let access_point = dbus::AccessPointProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        self.path = Some(path);

        let ssid = access_point.ssid().await?;
        let ssid_sub = access_point
            .receive_ssid_changed()
            .await
            .filter_map(|e| async move {
                let ssid = e.get().await.ok()?;
                Some(NetworkManagerEvent::Ssid(ssid))
            });

        let strength = access_point.strength().await?;
        let strength_sub =
            access_point
                .receive_strength_changed()
                .await
                .filter_map(|e| async move {
                    let strength = e.get().await.ok()?;
                    Some(NetworkManagerEvent::Strength(strength))
                });

        stream_map.emit(NetworkManagerEvent::Ssid(ssid))?;
        stream_map.emit(NetworkManagerEvent::Strength(strength))?;

        stream_map.add(SSID_CHANGED, ssid_sub);
        stream_map.add(STRENGTH_CHANGED, strength_sub);

        Ok(())
    }

    fn as_wifi_status_event(&self) -> Option<NetworkEvent> {
        let (ssid, strength) = self.ssid.as_ref().zip(self.strength.as_ref())?;
        Some(NetworkEvent::WifiStatus(WifiStatusEvent {
            wifi_status: COption::Some(WifiStatus {
                ssid: ssid.clone().into(),
                strength: *strength,
            }),
        }))
    }

    pub(crate) fn ssid_changed(&mut self, ssid: Vec<u8>) -> Option<NetworkEvent> {
        let ssid = String::from_utf8_lossy(&ssid).to_string();
        self.ssid = Some(ssid);
        self.as_wifi_status_event()
    }

    pub(crate) fn strength_changed(&mut self, strength: u8) -> Option<NetworkEvent> {
        self.strength = Some(strength);
        self.as_wifi_status_event()
    }
}
