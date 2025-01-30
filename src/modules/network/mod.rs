use crate::{
    dbus::{nm::NetworkManager, OrgFreedesktopNetworkManagerStateChanged},
    hyprctl,
    scheduler::{Module, RepeatingModule},
    Command,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs};
use std::time::Duration;

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network {
    conn: Connection,
}

impl Module for Network {
    const NAME: &str = "Network";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        let conn = Connection::new_system().context("Failed to connect to D-Bus")?;

        full_reset(&conn)?;

        conn.add_match(
            OrgFreedesktopNetworkManagerStateChanged::match_rule(None, None),
            |_: OrgFreedesktopNetworkManagerStateChanged, conn, _| {
                if let Err(err) = full_reset(conn) {
                    log::error!("{:?}", err);
                }
                true
            },
        )
        .context("failed to add_match")?;

        Ok(Some(Box::new(Network { conn })))
    }
}

impl RepeatingModule for Network {
    fn tick(&mut self) -> Result<Duration> {
        while self.conn.process(Duration::from_millis(200))? {}
        network_speed::update(&self.conn)?;

        Ok(Duration::from_secs(1))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
        if let Command::SpawnNetworkEditor = cmd {
            hyprctl::dispatch("exec kitty --name nmtui nmtui")?;
        }

        Ok(())
    }
}

fn full_reset(conn: &Connection) -> Result<()> {
    network_list::reset(conn)?;
    wifi_status::reset(conn);
    network_speed::reset();
    network_speed::update(conn)?;
    NetworkManager::primary_wireless_device(conn)?.set_refresh_rate_in_ms(conn, 1_000)?;
    Ok(())
}
