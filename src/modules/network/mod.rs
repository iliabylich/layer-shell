use crate::{
    dbus::{
        nm::{Device, NetworkManager},
        OrgFreedesktopNetworkManagerStateChanged,
    },
    hyprctl, Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs,
};
use std::{os::fd::AsRawFd, time::Duration};

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network {
    conn: Connection,
    fd: i32,
    tx: VerboseSender<Event>,
    primary_device: Option<Device>,
    transmitted_bytes: Option<u64>,
    received_bytes: Option<u64>,
}

impl Network {
    pub(crate) const INTERVAL: u64 = 1;

    pub(crate) fn new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel = Channel::get_private(BusType::System).unwrap();
        channel.set_watch_enabled(true);
        let fd = channel.watch().fd;
        let conn = Connection::from(channel);

        conn.add_match(
            OrgFreedesktopNetworkManagerStateChanged::match_rule(None, None),
            |_: OrgFreedesktopNetworkManagerStateChanged, _, _| true,
        )
        .context("failed to add_match")?;

        let mut this = Self {
            conn,
            fd,
            tx,
            primary_device: None,
            transmitted_bytes: None,
            received_bytes: None,
        };
        this.full_reset();
        Ok(this)
    }

    pub(crate) fn tick(&mut self) {
        self.update_network_speed();
    }

    pub(crate) fn read(&mut self) {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            if OrgFreedesktopNetworkManagerStateChanged::from_message(&message).is_some() {
                self.full_reset();
            }
        }
    }

    fn full_reset(&mut self) {
        match NetworkManager::primary_wireless_device(&self.conn) {
            Ok(primary_device) => {
                if let Err(err) = primary_device.set_refresh_rate_in_ms(&self.conn, 1_000) {
                    log::error!("failed to set refresh rate on primary device: {:?}", err);
                }
                self.primary_device = Some(primary_device);

                self.reset_network_list();
                self.reset_wifi_status();
                self.reset_network_speed();
                self.update_network_speed();
            }

            Err(err) => {
                log::error!("No primary network device: {:?}", err)
            }
        }
    }

    pub(crate) fn spawn_network_editor(&self) {
        if let Err(err) = hyprctl::dispatch("exec kitty --name nmtui nmtui") {
            log::error!("{:?}", err);
        }
    }
}

impl AsRawFd for Network {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd
    }
}
