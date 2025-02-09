use crate::{
    dbus::{nm::NetworkManager, OrgFreedesktopNetworkManagerStateChanged},
    hyprctl,
    scheduler::Actor,
    Command, Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs};
use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network {
    conn: Connection,
    tx: Sender<Event>,
}

impl Actor for Network {
    fn name() -> &'static str {
        "Network"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        let conn = Connection::new_system().context("Failed to connect to D-Bus")?;

        full_reset(&conn, &tx)?;

        {
            let tx = tx.clone();
            conn.add_match(
                OrgFreedesktopNetworkManagerStateChanged::match_rule(None, None),
                move |_: OrgFreedesktopNetworkManagerStateChanged, conn, _| {
                    if let Err(err) = full_reset(conn, &tx) {
                        log::error!("{:?}", err);
                    }
                    true
                },
            )
            .context("failed to add_match")?;
        }

        Ok(Box::new(Network { conn, tx }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while self.conn.process(Duration::from_millis(0))? {}
        network_speed::update(&self.conn, &self.tx)?;
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        if let Command::SpawnNetworkEditor = cmd {
            hyprctl::dispatch("exec kitty --name nmtui nmtui")?;
        }

        Ok(ControlFlow::Continue(()))
    }
}

fn full_reset(conn: &Connection, tx: &Sender<Event>) -> Result<()> {
    network_list::reset(conn, &tx)?;
    wifi_status::reset(conn, &tx)?;
    network_speed::reset();
    network_speed::update(conn, &tx)?;
    NetworkManager::primary_wireless_device(conn)?.set_refresh_rate_in_ms(conn, 1_000)?;
    Ok(())
}

impl std::fmt::Debug for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Network").field("conn", &"<conn>").finish()
    }
}
