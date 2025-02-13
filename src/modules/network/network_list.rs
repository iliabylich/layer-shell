use crate::{
    dbus::nm::{Device, NetworkManager},
    event::Network as NetworkData,
    modules::network::Network,
    Event,
};
use anyhow::Result;

impl Network {
    pub(crate) fn reset_network_list(&self) {
        let event = Event::NetworkList {
            list: self.get_network_list().unwrap_or_default().into(),
        };
        self.tx.send(event);
    }

    fn get_network_list(&self) -> Result<Vec<NetworkData>> {
        let mut ifaces = vec![];

        let devices = NetworkManager::get_devices(&self.conn)?;

        for device in devices {
            match self.get_network_for_device(&device) {
                Ok(network) => ifaces.push(network),
                Err(_) => log::warn!("Failed to get data for Device {device:?} (not connected?)"),
            }
        }

        Ok(ifaces)
    }

    fn get_network_for_device(&self, device: &Device) -> Result<NetworkData> {
        let iface = device.interface(&self.conn)?;
        let ip4_config = device.ip4_config(&self.conn)?;
        let address = ip4_config.address(&self.conn)?;

        Ok(NetworkData {
            iface: iface.into(),
            address: address.into(),
        })
    }
}
