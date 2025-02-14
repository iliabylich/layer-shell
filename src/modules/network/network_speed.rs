use crate::Event;

#[derive(Debug)]
enum OneWaySpeed {
    Uninitialized,
    Initialized { prev: u64 },
    InitializedWithError { prev: u64, err: u64 },
}

impl OneWaySpeed {
    fn update(&mut self, current: u64) -> u64 {
        match self {
            Self::Uninitialized => {
                *self = Self::Initialized { prev: current };
                0
            }
            Self::Initialized { prev } => {
                let d = current - *prev;
                *self = Self::InitializedWithError {
                    prev: current,
                    err: d,
                };
                d
            }
            Self::InitializedWithError { prev, err } => {
                let mut d = current - *prev;
                if d < *err && d != 0 {
                    *err = d;
                }
                d -= *err;
                *prev = current;
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
            upload_speed: fmt(upload_speed).into(),
            download_speed: fmt(download_speed).into(),
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
