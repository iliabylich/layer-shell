use crate::{ffi::CString, modules::network::Network, Event};

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

impl Network {
    pub(crate) fn reset_network_speed(&mut self) {
        self.transmitted_bytes = None;
        self.received_bytes = None;
    }

    pub(crate) fn update_network_speed(&mut self) {
        let Some(device) = self.primary_device.as_ref() else {
            return;
        };

        let tx_bytes = device
            .tx_bytes(&self.conn)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(0);
        let upload_speed = update_stat_and_return_delta(&mut self.transmitted_bytes, tx_bytes);

        let rx_bytes = device
            .rx_bytes(&self.conn)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(0);
        let download_speed = update_stat_and_return_delta(&mut self.received_bytes, rx_bytes);

        let event = Event::NetworkSpeed {
            upload_speed: format_speed(upload_speed),
            download_speed: format_speed(download_speed),
        };
        self.tx.send(event);
    }
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
