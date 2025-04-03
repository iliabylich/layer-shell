use crate::{
    Event, VerboseSender,
    dbus::{
        OrgFreedesktopNetworkManagerStateChanged,
        nm::{Device, NetworkManager},
    },
    epoll::{FdId, Reader},
    hyprctl,
    modules::maybe_connected::MaybeConnected,
};
use anyhow::{Context as _, Result};
use dbus::{
    arg::RefArg,
    blocking::{Connection, stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged},
    channel::{BusType, Channel},
    message::SignalArgs,
};
use network_speed::NetworkSpeed;
use std::{os::fd::RawFd, time::Duration};

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network {
    conn: Connection,
    tx: VerboseSender<Event>,
    primary_device: Option<Device>,
    network_speed: NetworkSpeed,
}

impl Network {
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

    pub(crate) fn new(tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(tx))
    }

    pub(crate) fn spawn_network_editor(&self) -> Result<()> {
        hyprctl::dispatch("exec wezterm start --always-new-process --class org.me.nmtui -e nmtui")
            .context("failed to spawn network editor")
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
}

impl Reader for Network {
    type Output = ();

    const NAME: &str = "Network";

    fn read(&mut self) -> Result<Self::Output> {
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

        Ok(())
    }

    fn fd(&self) -> RawFd {
        self.conn.channel().watch().fd
    }

    fn fd_id(&self) -> FdId {
        FdId::NetworkDBus
    }
}
