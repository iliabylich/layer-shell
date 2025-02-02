use crate::{
    dbus::{nm::NetworkManager, OrgFreedesktopNetworkManagerStateChanged},
    hyprctl,
    scheduler::Actor,
    Command,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs};
use std::{ops::ControlFlow, time::Duration};

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network {
    conn: Connection,
}

impl Actor for Network {
    fn name() -> &'static str {
        "Network"
    }

    fn start() -> Result<Box<dyn Actor>> {
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

        Ok(Box::new(Network { conn }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while self.conn.process(Duration::from_millis(200))? {}
        network_speed::update(&self.conn)?;
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        if let Command::SpawnNetworkEditor = cmd {
            hyprctl::dispatch("exec kitty --name nmtui nmtui")?;
        }

        Ok(ControlFlow::Continue(()))
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

impl std::fmt::Debug for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Network").field("conn", &"<conn>").finish()
    }
}
