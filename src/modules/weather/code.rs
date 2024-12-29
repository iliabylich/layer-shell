#[derive(Debug, Clone, Copy)]
#[repr(C)]
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
#[repr(C)]
pub enum Fog {
    FogNormal,
    FogDepositingRime,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Drizzle {
    DrizzleLight,
    DrizzleModerate,
    DrizzleDense,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum FreezingDrizzle {
    FreezingDrizzleLight,
    FreezingDrizzleDense,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Rain {
    RainSlight,
    RainModerate,
    RainHeavy,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum FreezingRain {
    FreezingRainLight,
    FreezingRainHeavy,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum SnowFall {
    SnowFallSlight,
    SnowFallModerate,
    SnowFallHeavy,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum RainShowers {
    RainShowersSlight,
    RainShowersModerate,
    RainShowersViolent,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum SnowShowers {
    SnowShowersSlight,
    SnowShowersHeavy,
}
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum ThunderstormWithHail {
    ThunderstormWithHailSight,
    ThunderstormWithHailHeavy,
}
