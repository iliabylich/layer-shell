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

impl From<u32> for WeatherCode {
    fn from(value: u32) -> Self {
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
