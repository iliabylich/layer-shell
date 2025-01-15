use crate::{dbus::nm::NetworkManager, global::global, Event};
use anyhow::Result;
use dbus::blocking::Connection;

global!(TRANSMITTED_BYTES, Option<u64>);
global!(RECEIVED_BYTES, Option<u64>);

fn update_stat_and_return_delta(stat: &'static mut Option<u64>, value: u64) -> u64 {
    match stat {
        Some(prev) => {
            let delta = value - *prev;
            *prev = value;
            delta
        }
        None => {
            *stat = Some(value);
            0
        }
    }
}

pub(crate) fn reset() {
    TRANSMITTED_BYTES::set(None);
    RECEIVED_BYTES::set(None);
}

pub(crate) fn update(conn: &Connection) -> Result<()> {
    let device = NetworkManager::primary_wireless_device(conn)?;

    let tx_bytes = device
        .tx_bytes(conn)
        .inspect_err(|err| log::error!("{:?}", err))
        .unwrap_or(0);
    let upload_speed = update_stat_and_return_delta(TRANSMITTED_BYTES::get(), tx_bytes);

    let rx_bytes = device
        .rx_bytes(conn)
        .inspect_err(|err| log::error!("{:?}", err))
        .unwrap_or(0);
    let download_speed = update_stat_and_return_delta(RECEIVED_BYTES::get(), rx_bytes);

    let event = Event::NetworkSpeed {
        upload_speed,
        download_speed,
    };
    event.emit();

    Ok(())
}
