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
    const INTERVAL: Option<u64> = Some(1000);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
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

        Ok(Box::new(conn))
    }

    fn tick(state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let conn = state
            .downcast_ref::<Connection>()
            .context("Network state is malformed")?;

        conn.process(std::time::Duration::from_millis(200))?;
        network_speed::update(conn)?;
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
