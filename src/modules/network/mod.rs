use crate::{
    dbus::{nm::NetworkManager, OrgFreedesktopNetworkManagerStateChanged},
    scheduler::Module,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs};
use std::any::Any;

mod network_list;
mod network_speed;
mod wifi_status;

pub(crate) struct Network;

impl Module for Network {
    const NAME: &str = "Network";
    const INTERVAL: Option<u64> = None;

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        let conn = Connection::new_system().context("Failed to connect to D-Bus")?;

        std::thread::spawn(move || {
            if let Err(err) = in_thread(&conn) {
                log::error!("{:?}", err);
            }
        });

        Ok(Box::new(2))
    }

    fn tick(_: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        unreachable!()
    }
}

fn in_thread(conn: &Connection) -> Result<()> {
    full_reset(conn)?;

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

    loop {
        if let Err(err) = spin(conn) {
            log::error!("{:?}", err);
        }
    }
}

fn spin(conn: &Connection) -> Result<()> {
    conn.process(std::time::Duration::from_millis(1000))?;
    network_speed::update(conn)?;
    Ok(())
}

fn full_reset(conn: &Connection) -> Result<()> {
    network_list::reset(conn)?;
    wifi_status::reset(conn);
    network_speed::reset();
    network_speed::update(conn)?;
    NetworkManager::primary_wireless_device(conn)?.set_refresh_rate_in_ms(conn, 1_000)?;
    Ok(())
}
