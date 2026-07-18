use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{getenv, new_sockaddr_un},
};
use anyhow::{Context, Result, bail};
use libc::sockaddr_un;

pub(crate) struct Weather {
    reader: Box<UnixSocketReader>,
    buf: Buffer,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ProxyUntilRead,
    WaitingForWrite(Wants),
    WriteFinished(Wants),
    Proxy,
}

pub const HOURLY_WEATHER_FORECAST_LENGTH: usize = 10;
pub const DAILY_WEATHER_FORECAST_LENGTH: usize = 6;

impl Weather {
    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/weather-mon.sock");
        let addr = new_sockaddr_un(path.as_bytes())?;
        Ok(addr)
    }

    pub(crate) fn new() -> Self {
        Self {
            reader: Box::new(UnixSocketReader::new()),
            buf: Buffer::new(),
            state: State::ProxyUntilRead,
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        match self.state {
            State::ProxyUntilRead => {
                let wants = self.reader.wants(addr);
                if let Some(wants @ Wants::Read { fd, .. }) = wants {
                    self.state = State::WaitingForWrite(wants);
                    Some(Wants::Write {
                        fd,
                        buf: b"1".as_ptr(),
                        len: 1,
                    })
                } else {
                    wants
                }
            }
            State::WaitingForWrite(_) => None,
            State::WriteFinished(wants) => {
                self.state = State::Proxy;
                Some(wants)
            }
            State::Proxy => self.reader.wants(addr),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        match satisfy {
            Satisfy::Socket(res) => {
                let fd = res?;
                self.reader.satisfy_socket(fd)?;
                Ok(())
            }

            Satisfy::Connect(res) => {
                res?;
                self.reader.satisfy_connect()?;
                Ok(())
            }

            Satisfy::Write(res) => {
                res?;
                if let State::WaitingForWrite(wants) = self.state {
                    self.state = State::WriteFinished(wants);
                } else {
                    bail!("malformed state");
                }
                Ok(())
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let (buf, len) = self.reader.satisfy_read(bytes_read)?;
                let bytes = buf.get(..len).context("buf is too short")?;

                for event in self.buf.push(bytes) {
                    events.push_back(Event::Weather {
                        temperature: event.current.t,
                        code: event.current.code,
                        hourly_forecast: event.hourly,
                        daily_forecast: event.daily,
                    });
                }

                Ok(())
            }

            _ => bail!("Weather only accepts Socket, Connect and Read, got: {satisfy:?}"),
        }
    }
}

struct Buffer(Vec<u8>);
impl Buffer {
    const fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, bytes: &[u8]) -> Vec<WeatherData> {
        self.0.extend_from_slice(bytes);
        let mut events = vec![];

        while let Some((first, rest)) = self.0.split_first_chunk::<{ WeatherData::BYTESIZE }>() {
            let weather = WeatherData::deserialize(first);
            events.push(weather);
            self.0 = rest.to_vec();
        }

        events
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WeatherData {
    current: CurrentWeather,
    hourly: [WeatherOnHour; HOURLY_WEATHER_FORECAST_LENGTH],
    daily: [WeatherOnDay; DAILY_WEATHER_FORECAST_LENGTH],
}
#[derive(Clone, Copy, PartialEq)]
struct CurrentWeather {
    t: f32,
    code: WeatherCode,
}
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct WeatherOnHour {
    pub unix_seconds: i64,
    pub temperature: f32,
    pub code: WeatherCode,
}
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct WeatherOnDay {
    pub unix_seconds: i64,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}

impl core::fmt::Debug for CurrentWeather {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} - {:?}", self.t, self.code)
    }
}
impl core::fmt::Debug for WeatherOnHour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} - {} - {:?}",
            self.unix_seconds, self.temperature, self.code
        )
    }
}
impl core::fmt::Debug for WeatherOnDay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} - {}..{} - {:?}",
            self.unix_seconds, self.temperature_min, self.temperature_max, self.code
        )
    }
}

impl CurrentWeather {
    const BYTESIZE: usize = 8;

    const fn deserialize(buf: [u8; Self::BYTESIZE]) -> Self {
        Self {
            t: f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            code: WeatherCode::deserialize(buf[4]),
        }
    }
}
impl WeatherOnHour {
    const BYTESIZE: usize = 16;

    const fn deserialize(buf: [u8; Self::BYTESIZE]) -> Self {
        Self {
            unix_seconds: i64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]),
            temperature: f32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            code: WeatherCode::deserialize(buf[12]),
        }
    }
}
impl WeatherOnDay {
    const BYTESIZE: usize = 24;

    const fn deserialize(buf: [u8; Self::BYTESIZE]) -> Self {
        Self {
            unix_seconds: i64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]),
            temperature_min: f32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            temperature_max: f32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            code: WeatherCode::deserialize(buf[16]),
        }
    }
}
impl WeatherData {
    const BYTESIZE: usize = (CurrentWeather::BYTESIZE
        + WeatherOnHour::BYTESIZE * HOURLY_WEATHER_FORECAST_LENGTH
        + WeatherOnDay::BYTESIZE * DAILY_WEATHER_FORECAST_LENGTH);

    #[expect(clippy::arithmetic_side_effects)]
    fn deserialize(buf: &[u8; Self::BYTESIZE]) -> Self {
        struct Cursor<'a, const N: usize> {
            buf: &'a [u8; N],
            offset: usize,
        }
        impl<const N: usize> Cursor<'_, N> {
            fn take<const M: usize>(&mut self) -> [u8; M] {
                let mut out = [0; M];
                out.copy_from_slice(&self.buf[self.offset..self.offset + M]);
                self.offset += M;
                out
            }
        }

        let mut cursor = Cursor { buf, offset: 0 };

        Self {
            current: CurrentWeather::deserialize(cursor.take()),
            hourly: core::array::from_fn(|_| WeatherOnHour::deserialize(cursor.take())),
            daily: core::array::from_fn(|_| WeatherOnDay::deserialize(cursor.take())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum WeatherCode {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,

    FogNormal,
    FogDepositingRime,

    DrizzleLight,
    DrizzleModerate,
    DrizzleDense,

    FreezingDrizzleLight,
    FreezingDrizzleDense,

    RainSlight,
    RainModerate,
    RainHeavy,

    FreezingRainLight,
    FreezingRainHeavy,

    SnowFallSlight,
    SnowFallModerate,
    SnowFallHeavy,

    SnowGrains,

    RainShowersSlight,
    RainShowersModerate,
    RainShowersViolent,

    SnowShowersSlight,
    SnowShowersHeavy,

    Thunderstorm,

    ThunderstormWithHailSight,
    ThunderstormWithHailHeavy,

    Unknown,
}

impl WeatherCode {
    const fn deserialize(value: u8) -> Self {
        match value {
            0 => Self::ClearSky,
            1 => Self::MainlyClear,
            2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 => Self::FogNormal,
            48 => Self::FogDepositingRime,
            51 => Self::DrizzleLight,
            53 => Self::DrizzleModerate,
            55 => Self::DrizzleDense,
            56 => Self::FreezingDrizzleLight,
            57 => Self::FreezingDrizzleDense,
            61 => Self::RainSlight,
            63 => Self::RainModerate,
            65 => Self::RainHeavy,
            66 => Self::FreezingRainLight,
            67 => Self::FreezingRainHeavy,
            71 => Self::SnowFallSlight,
            73 => Self::SnowFallModerate,
            75 => Self::SnowFallHeavy,
            77 => Self::SnowGrains,
            80 => Self::RainShowersSlight,
            81 => Self::RainShowersModerate,
            82 => Self::RainShowersViolent,
            85 => Self::SnowShowersSlight,
            86 => Self::SnowShowersHeavy,
            95 => Self::Thunderstorm,
            96 => Self::ThunderstormWithHailSight,
            99 => Self::ThunderstormWithHailHeavy,
            _ => Self::Unknown,
        }
    }
}
