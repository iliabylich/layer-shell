use crate::Event;

#[derive(Debug)]
enum OneWaySpeed {
    Unset,
    Set(u64),
}

impl OneWaySpeed {
    const THRESHOLD: u64 = 5_000;

    fn update(&mut self, current: u64) -> u64 {
        match self {
            Self::Unset => {
                *self = Self::Set(current);
                0
            }
            Self::Set(prev) => {
                let mut d = current.saturating_sub(*prev);
                *self = Self::Set(current);
                if d < Self::THRESHOLD {
                    d = 0;
                }
                d
            }
        }
    }
}

pub(crate) struct Speed {
    // transmitted
    tx: OneWaySpeed,
    // received
    rx: OneWaySpeed,
}

impl Speed {
    pub(crate) fn new() -> Self {
        Self {
            tx: OneWaySpeed::Unset,
            rx: OneWaySpeed::Unset,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.tx = OneWaySpeed::Unset;
        self.rx = OneWaySpeed::Unset;
    }

    pub(crate) fn update_tx(&mut self, tx: u64) -> Event {
        let d = self.tx.update(tx);
        Event::UploadSpeed { bytes_per_sec: d }
    }

    pub(crate) fn update_rx(&mut self, rx: u64) -> Event {
        let d = self.rx.update(rx);
        Event::DownloadSpeed { bytes_per_sec: d }
    }
}
