#[derive(Debug, Clone, Copy)]
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
