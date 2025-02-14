use dbus::blocking::Connection;

use crate::{dbus::nm::Device, Event};

pub(crate) struct NetworkSpeed {
    transmitted: Option<u64>,
    received: Option<u64>,
}

impl NetworkSpeed {
    pub(crate) fn new() -> Self {
        Self {
            transmitted: None,
            received: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.transmitted = None;
        self.received = None;
    }

    pub(crate) fn update(&mut self, device: &Device, conn: &Connection) -> Event {
        fn update_stat_and_return_delta(stat: &mut Option<u64>, value: u64) -> u64 {
            match *stat {
                Some(prev) => {
                    let delta = value - prev;
                    *stat = Some(value);
                    delta
                }
                None => {
                    *stat = Some(value);
                    0
                }
            }
        }

        let tx_bytes = device
            .tx_bytes(conn)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(0);
        let upload_speed = update_stat_and_return_delta(&mut self.transmitted, tx_bytes);

        let rx_bytes = device
            .rx_bytes(conn)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(0);
        let download_speed = update_stat_and_return_delta(&mut self.received, rx_bytes);

        Event::NetworkSpeed {
            upload_speed: format_network_speed(upload_speed).into(),
            download_speed: format_network_speed(download_speed).into(),
        }
    }
}

fn format_network_speed(mut speed: u64) -> String {
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
}
