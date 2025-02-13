use crate::{event::WifiStatus, ffi::COption, modules::network::Network, Event};
use anyhow::{bail, Result};

impl Network {
    pub(crate) fn reset_wifi_status(&self) {
        let wifi_status = match self.get_wifi_status() {
            Ok((ssid, strength)) => COption::Some(WifiStatus {
                ssid: ssid.into(),
                strength,
            }),
            Err(err) => {
                log::warn!("WiFiStatus error: {:?}", err);
                COption::None
            }
        };

        let event = Event::WifiStatus { wifi_status };
        self.tx.send(event);
    }

    fn get_wifi_status(&self) -> Result<(String, u8)> {
        let Some(device) = self.primary_device.as_ref() else {
            bail!("no primary device");
        };
        let access_point = device.active_access_point(&self.conn)?;
        let ssid = access_point.ssid(&self.conn)?;
        let strength = access_point.strength(&self.conn)?;

        Ok((ssid, strength))
    }
}
