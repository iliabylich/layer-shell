use std::fmt::{self, Display};

#[derive(Debug)]
pub(crate) enum WeatherCode {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog(Fog),
    Drizzle(Drizzle),
    FreezingDrizzle(FreezingDrizzle),
    Rain(Rain),
    FreezingRain(FreezingRain),
    SnowFall(SnowFall),
    SnowGrains,
    RainShowers(RainShowers),
    SnowShowers(SnowShowers),
    Thunderstorm,
    ThunderstormWithHail(ThunderstormWithHail),
    Unknown,
}

#[derive(Debug)]
pub(crate) enum Fog {
    Normal,
    DepositingRime,
}
#[derive(Debug)]
pub(crate) enum Drizzle {
    Light,
    Moderate,
    Dense,
}
#[derive(Debug)]
pub(crate) enum FreezingDrizzle {
    Light,
    Dense,
}
#[derive(Debug)]
pub(crate) enum Rain {
    Slight,
    Moderate,
    Heavy,
}
#[derive(Debug)]
pub(crate) enum FreezingRain {
    Light,
    Heavy,
}
#[derive(Debug)]
pub(crate) enum SnowFall {
    Slight,
    Moderate,
    Heavy,
}

#[derive(Debug)]
pub(crate) enum RainShowers {
    Slight,
    Moderate,
    Violent,
}
#[derive(Debug)]
pub(crate) enum SnowShowers {
    Slight,
    Heavy,
}
#[derive(Debug)]
pub(crate) enum ThunderstormWithHail {
    Sight,
    Heavy,
}

impl From<u32> for WeatherCode {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::ClearSky,
            1 => Self::MainlyClear,
            2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 => Self::Fog(Fog::Normal),
            48 => Self::Fog(Fog::DepositingRime),
            51 => Self::Drizzle(Drizzle::Light),
            53 => Self::Drizzle(Drizzle::Moderate),
            55 => Self::Drizzle(Drizzle::Dense),
            56 => Self::FreezingDrizzle(FreezingDrizzle::Light),
            57 => Self::FreezingDrizzle(FreezingDrizzle::Dense),
            61 => Self::Rain(Rain::Slight),
            63 => Self::Rain(Rain::Moderate),
            65 => Self::Rain(Rain::Heavy),
            66 => Self::FreezingRain(FreezingRain::Light),
            67 => Self::FreezingRain(FreezingRain::Heavy),
            71 => Self::SnowFall(SnowFall::Slight),
            73 => Self::SnowFall(SnowFall::Moderate),
            75 => Self::SnowFall(SnowFall::Heavy),
            77 => Self::SnowGrains,
            80 => Self::RainShowers(RainShowers::Slight),
            81 => Self::RainShowers(RainShowers::Moderate),
            82 => Self::RainShowers(RainShowers::Violent),
            85 => Self::SnowShowers(SnowShowers::Slight),
            86 => Self::SnowShowers(SnowShowers::Heavy),
            95 => Self::Thunderstorm,
            96 => Self::ThunderstormWithHail(ThunderstormWithHail::Sight),
            99 => Self::ThunderstormWithHail(ThunderstormWithHail::Heavy),
            _ => Self::Unknown,
        }
    }
}

impl WeatherCode {
    pub(crate) fn icon(&self) -> char {
        match self {
            WeatherCode::ClearSky | WeatherCode::MainlyClear => '',
            WeatherCode::PartlyCloudy | WeatherCode::Overcast => '',
            WeatherCode::Fog(_) => '',
            WeatherCode::Drizzle(_) | WeatherCode::FreezingDrizzle(_) => '',
            WeatherCode::Rain(_) | WeatherCode::FreezingRain(_) => '',
            WeatherCode::SnowFall(_) | WeatherCode::SnowGrains => '',
            WeatherCode::RainShowers(_) => '',
            WeatherCode::SnowShowers(_) => '',
            WeatherCode::Thunderstorm | WeatherCode::ThunderstormWithHail(_) => '',
            WeatherCode::Unknown => '?',
        }
    }
}

impl Display for WeatherCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WeatherCode::*;
        match self {
            ClearSky => write!(f, "Clear Sky"),
            MainlyClear => write!(f, "Mainly Clear"),
            PartlyCloudy => write!(f, "Partly Cloudy"),
            Overcast => write!(f, "Overcast"),
            Fog(inner) => write!(f, "Fog ({})", inner),
            Drizzle(inner) => write!(f, "Drizzle ({})", inner),
            FreezingDrizzle(inner) => write!(f, "Freezing Drizzle ({})", inner),
            Rain(inner) => write!(f, "Rain ({})", inner),
            FreezingRain(inner) => write!(f, "Freezing Rain ({})", inner),
            SnowFall(inner) => write!(f, "Snow Fall ({})", inner),
            SnowGrains => write!(f, "Snow Grains"),
            RainShowers(inner) => write!(f, "Rain Showers ({})", inner),
            SnowShowers(inner) => write!(f, "Snow Showers ({})", inner),
            Thunderstorm => write!(f, "Thunderstorm"),
            ThunderstormWithHail(inner) => write!(f, "Thunderstorm With Hail({})", inner),
            Unknown => write!(f, "Unknown"),
        }
    }
}

impl Display for Fog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fog::Normal => write!(f, "Normal"),
            Fog::DepositingRime => write!(f, "Depositing Rime"),
        }
    }
}

impl Display for Drizzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Drizzle::Light => write!(f, "Light"),
            Drizzle::Moderate => write!(f, "Moderate"),
            Drizzle::Dense => write!(f, "Dense"),
        }
    }
}

impl Display for FreezingDrizzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FreezingDrizzle::Light => write!(f, "Light"),
            FreezingDrizzle::Dense => write!(f, "Dense"),
        }
    }
}

impl Display for Rain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rain::Slight => write!(f, "Slight"),
            Rain::Moderate => write!(f, "Moderate"),
            Rain::Heavy => write!(f, "Heavy"),
        }
    }
}

impl Display for FreezingRain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FreezingRain::Light => write!(f, "Light"),
            FreezingRain::Heavy => write!(f, "Heavy"),
        }
    }
}

impl Display for SnowFall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnowFall::Slight => write!(f, "Slight"),
            SnowFall::Moderate => write!(f, "Moderate"),
            SnowFall::Heavy => write!(f, "Heavy"),
        }
    }
}

impl Display for RainShowers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RainShowers::Slight => write!(f, "Slight"),
            RainShowers::Moderate => write!(f, "Moderate"),
            RainShowers::Violent => write!(f, "Violent"),
        }
    }
}

impl Display for SnowShowers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnowShowers::Slight => write!(f, "Slight"),
            SnowShowers::Heavy => write!(f, "Heavy"),
        }
    }
}

impl Display for ThunderstormWithHail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThunderstormWithHail::Sight => write!(f, "Sight"),
            ThunderstormWithHail::Heavy => write!(f, "Heavy"),
        }
    }
}
