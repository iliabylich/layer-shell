use crate::{dbus::nm::NetworkManager, ffi::CString, Event};
use anyhow::Result;
use dbus::blocking::Connection;
use std::sync::atomic::{AtomicU64, Ordering};

static TRANSMITTED_BYTES: AtomicU64 = AtomicU64::new(u64::MAX);
static RECEIVED_BYTES: AtomicU64 = AtomicU64::new(u64::MAX);

fn update_stat_and_return_delta(stat: &AtomicU64, value: u64) -> u64 {
    let prev = stat.load(Ordering::Relaxed);
    if prev == u64::MAX {
        stat.store(value, Ordering::Relaxed);
        0
    } else {
        let delta = value - prev;
        stat.store(value, Ordering::Relaxed);
        delta
    }
}

pub(crate) fn reset() {
    TRANSMITTED_BYTES.store(u64::MAX, Ordering::Relaxed);
    RECEIVED_BYTES.store(u64::MAX, Ordering::Relaxed);
}

pub(crate) fn update(conn: &Connection) -> Result<()> {
    let device = NetworkManager::primary_wireless_device(conn)?;

    let tx_bytes = device
        .tx_bytes(conn)
        .inspect_err(|err| log::error!("{:?}", err))
        .unwrap_or(0);
    let upload_speed = update_stat_and_return_delta(&TRANSMITTED_BYTES, tx_bytes);

    let rx_bytes = device
        .rx_bytes(conn)
        .inspect_err(|err| log::error!("{:?}", err))
        .unwrap_or(0);
    let download_speed = update_stat_and_return_delta(&RECEIVED_BYTES, rx_bytes);

    let event = Event::NetworkSpeed {
        upload_speed: format_speed(upload_speed),
        download_speed: format_speed(download_speed),
    };
    event.emit();

    Ok(())
}

fn format_speed(mut speed: u64) -> CString {
    enum Unit {
        B,
        KB,
        MB,
    }

    let mut units = Unit::B;

    if speed / 1_024 > 0 {
        speed /= 1024;
        units = Unit::KB;
        if speed / 1_024 > 0 {
            speed /= 1024;
            units = Unit::MB;
        }
    }

    format!(
        "{speed}{}",
        match units {
            Unit::B => "B/s",
            Unit::KB => "KB/s",
            Unit::MB => "MB/s",
        }
    )
    .into()
}
