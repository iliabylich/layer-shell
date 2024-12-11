#[derive(Debug, Clone, Copy)]
pub enum Code {
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
