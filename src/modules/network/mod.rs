use crate::{
    dbus::{
        nm::{Device, NetworkManager},
        OrgFreedesktopNetworkManagerStateChanged,
    },
    hyprctl, Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::RefArg,
    blocking::{stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged, Connection},
    channel::{BusType, Channel},
    message::SignalArgs,
};
use network_speed::NetworkSpeed;
use std::time::Duration;

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct ConnectedNetwork {
    conn: Connection,
    tx: VerboseSender<Event>,
    primary_device: Option<Device>,
    network_speed: NetworkSpeed,
}

impl ConnectedNetwork {
    fn try_new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::System).context("failed to connecto to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);

        conn.add_match(
            OrgFreedesktopNetworkManagerStateChanged::match_rule(None, None),
            |_: OrgFreedesktopNetworkManagerStateChanged, _, _| true,
        )
        .context("failed to add_match")?;

        let mut this = Self {
            conn,
            tx,
            primary_device: None,
            network_speed: NetworkSpeed::new(),
        };
        this.full_reset();
        Ok(this)
    }

    fn read(&mut self) {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            if OrgFreedesktopNetworkManagerStateChanged::from_message(&message).is_some() {
                self.full_reset();
            } else if let Some(e) = PropertiesPropertiesChanged::from_message(&message) {
                if e.interface_name == "org.freedesktop.NetworkManager.Device.Statistics" {
                    let tx = e.changed_properties.get("TxBytes").and_then(|v| v.as_u64());
                    let rx = e.changed_properties.get("RxBytes").and_then(|v| v.as_u64());

                    if let Some((tx, rx)) = tx.zip(rx) {
                        let event = self.network_speed.update(tx, rx);
                        self.tx.send(event);
                    }
                }
            }
        }
    }

    fn full_reset(&mut self) {
        match NetworkManager::primary_wireless_device(&self.conn) {
            Ok(primary_device) => {
                if let Err(err) = primary_device.set_refresh_rate_in_ms(&self.conn, 1_000) {
                    log::error!("failed to set refresh rate on primary device: {:?}", err);
                }

                let event = network_list::load(&self.conn);
                self.tx.send(event);

                let event = wifi_status::load(&primary_device, &self.conn);
                self.tx.send(event);

                self.conn
                    .add_match_no_cb(
                        &PropertiesPropertiesChanged::match_rule(None, Some(&primary_device.path))
                            .match_str(),
                    )
                    .unwrap();

                self.network_speed.reset();

                self.primary_device = Some(primary_device);
            }

            Err(err) => {
                log::error!("No primary network device: {:?}", err)
            }
        }
    }

    fn fd(&self) -> i32 {
        self.conn.channel().watch().fd
    }
}

pub(crate) enum Network {
    Connected(ConnectedNetwork),
    Disconnected,
}

impl Network {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        ConnectedNetwork::try_new(tx)
            .inspect_err(|err| log::error!("{:?}", err))
            .map(Self::Connected)
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(network) = self {
            network.read();
        }
    }

    pub(crate) fn spawn_network_editor(&self) {
        if let Err(err) = hyprctl::dispatch("exec kitty --name nmtui nmtui") {
            log::error!("{:?}", err);
        }
    }

    pub(crate) fn fd(&mut self) -> Option<i32> {
        if let Self::Connected(network) = self {
            Some(network.fd())
        } else {
            None
        }
    }
}
