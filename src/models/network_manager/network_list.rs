use rusty_network_manager::{DeviceProxy, IP4ConfigProxy};

use crate::{models::NM, utils::singleton};

#[derive(Debug)]
pub(crate) struct Iface {
    pub(crate) name: String,
    pub(crate) ip: String,
}
#[derive(Debug)]
pub(crate) struct NetworkList {
    list: Vec<Iface>,
}
singleton!(NetworkList);

impl NetworkList {
    pub(crate) fn spawn() {
        Self::set(Self { list: vec![] });

        gtk4::glib::spawn_future_local(async {
            NM::start_if_not_started().await;

            loop {
                let ifaces = Self::get_state().await;
                Self::get().list = ifaces;

                gtk4::glib::timeout_future_seconds(5).await;
            }
        });
    }

    pub(crate) fn get_current() -> &'static [Iface] {
        &Self::get().list
    }

    async fn get_state() -> Vec<Iface> {
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
                    if let Some(address) = address_data.first().and_then(|addr| addr.get("address"))
                    {
                        let name = device.interface().await.unwrap();
                        let ip = address.downcast_ref::<String>().unwrap();
                        ifaces.push(Iface { name, ip });
                    }
                }
            }
        }

        ifaces
    }
}
