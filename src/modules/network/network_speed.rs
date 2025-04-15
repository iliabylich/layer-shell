use crate::Event;

#[derive(Debug)]
enum OneWaySpeed {
    Uninitialized,
    Initialized(u64),
}

const THREASHOLD: u64 = 5_000;

impl OneWaySpeed {
    fn update(&mut self, current: u64) -> u64 {
        match self {
            Self::Uninitialized => {
                *self = Self::Initialized(current);
                0
            }
            Self::Initialized(prev) => {
                let mut d = current - *prev;
                *self = Self::Initialized(current);
                if d < THREASHOLD {
                    d = 0;
                }
                d
            }
        }
    }
}

pub(crate) struct NetworkSpeed {
    // transmitted
    tx: OneWaySpeed,
    // received
    rx: OneWaySpeed,
}

impl NetworkSpeed {
    pub(crate) fn new() -> Self {
        Self {
            tx: OneWaySpeed::Uninitialized,
            rx: OneWaySpeed::Uninitialized,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.tx = OneWaySpeed::Uninitialized;
        self.rx = OneWaySpeed::Uninitialized;
    }

    pub(crate) fn update(&mut self, tx: u64, rx: u64) -> Event {
        let upload_speed = self.tx.update(tx);
        let download_speed = self.rx.update(rx);

        Event::NetworkSpeed {
            upload_speed: fmt(upload_speed),
            download_speed: fmt(download_speed),
        }
    }
}

fn fmt(mut speed: u64) -> String {
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
