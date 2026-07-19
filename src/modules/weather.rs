use crate::{
    Event,
    emitter::Emitter,
    sansio::{Satisfy, UnixSocketReader, Wants},
    utils::{FixedSizeBuffer, getenv, new_sockaddr_un},
};
use anyhow::{Context, Result};
use libc::sockaddr_un;

#[derive(Clone, Copy)]
pub(crate) struct Weather {
    reader: UnixSocketReader,
    emitter: Emitter,
}

pub const HOURLY_WEATHER_FORECAST_LENGTH: usize = 10;
pub const DAILY_WEATHER_FORECAST_LENGTH: usize = 6;

impl Weather {
    pub(crate) const BUFFER_SIZE: usize = WeatherData::BYTESIZE;

    pub(crate) fn address() -> Result<sockaddr_un> {
        let xdg_runtime_dir =
            core::str::from_utf8(getenv(c"XDG_RUNTIME_DIR").context("no $XDG_RUNTIME_DIR")?)?;
        let path = format!("{xdg_runtime_dir}/weather-mon.sock");
        let addr = new_sockaddr_un(path.as_bytes())?;
        Ok(addr)
    }

    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self {
            reader: UnixSocketReader::new(),
            emitter,
        }
    }

    pub(crate) fn wants(
        &mut self,
        addr: &sockaddr_un,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Option<Wants> {
        self.reader.wants(addr, buf.remainder())
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: &mut FixedSizeBuffer<{ Self::BUFFER_SIZE }>,
    ) -> Result<()> {
        if let Some(written) = self.reader.satisfy(satisfy)?
            && let Some(buf) = buf.written(written)
        {
            let event = WeatherData::deserialize(&buf);
            self.emitter.emit(&Event::Weather {
                temperature: event.current.t,
                code: event.current.code,
                hourly_forecast: event.hourly,
                daily_forecast: event.daily,
            });
        }

        Ok(())
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
