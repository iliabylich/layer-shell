#[derive(Debug, Clone, Copy)]
pub enum WeatherCode {
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

#[derive(Debug, Clone, Copy)]
pub enum Fog {
    Normal,
    DepositingRime,
}
#[derive(Debug, Clone, Copy)]
pub enum Drizzle {
    Light,
    Moderate,
    Dense,
}
#[derive(Debug, Clone, Copy)]
pub enum FreezingDrizzle {
    Light,
    Dense,
}
#[derive(Debug, Clone, Copy)]
pub enum Rain {
    Slight,
    Moderate,
    Heavy,
}
#[derive(Debug, Clone, Copy)]
pub enum FreezingRain {
    Light,
    Heavy,
}
#[derive(Debug, Clone, Copy)]
pub enum SnowFall {
    Slight,
    Moderate,
    Heavy,
}

#[derive(Debug, Clone, Copy)]
pub enum RainShowers {
    Slight,
    Moderate,
    Violent,
}
#[derive(Debug, Clone, Copy)]
pub enum SnowShowers {
    Slight,
    Heavy,
}
#[derive(Debug, Clone, Copy)]
pub enum ThunderstormWithHail {
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
