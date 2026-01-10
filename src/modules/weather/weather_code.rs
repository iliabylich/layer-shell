#[derive(Debug, Clone, Copy, PartialEq)]
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
            0 => WeatherCode::ClearSky,
            1 => WeatherCode::MainlyClear,
            2 => WeatherCode::PartlyCloudy,
            3 => WeatherCode::Overcast,
            45 => WeatherCode::FogNormal,
            48 => WeatherCode::FogDepositingRime,
            51 => WeatherCode::DrizzleLight,
            53 => WeatherCode::DrizzleModerate,
            55 => WeatherCode::DrizzleDense,
            56 => WeatherCode::FreezingDrizzleLight,
            57 => WeatherCode::FreezingDrizzleDense,
            61 => WeatherCode::RainSlight,
            63 => WeatherCode::RainModerate,
            65 => WeatherCode::RainHeavy,
            66 => WeatherCode::FreezingRainLight,
            67 => WeatherCode::FreezingRainHeavy,
            71 => WeatherCode::SnowFallSlight,
            73 => WeatherCode::SnowFallModerate,
            75 => WeatherCode::SnowFallHeavy,
            77 => WeatherCode::SnowGrains,
            80 => WeatherCode::RainShowersSlight,
            81 => WeatherCode::RainShowersModerate,
            82 => WeatherCode::RainShowersViolent,
            85 => WeatherCode::SnowShowersSlight,
            86 => WeatherCode::SnowShowersHeavy,
            95 => WeatherCode::Thunderstorm,
            96 => WeatherCode::ThunderstormWithHailSight,
            99 => WeatherCode::ThunderstormWithHailHeavy,
            _ => WeatherCode::Unknown,
        }
    }
}
