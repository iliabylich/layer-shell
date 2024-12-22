use crate::icons::{
    foggy_icon, partly_cloudy_icon, question_mark_icon, rainy_icon, snowy_icon, sunny_icon,
    thunderstorm_icon,
};
use layer_shell_io::weather::{
    Code, Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
    ThunderstormWithHail,
};

pub(crate) fn weather_code_to_icon(code: Code) -> &'static gtk4::gdk::Texture {
    match code {
        Code::ClearSky | Code::MainlyClear => sunny_icon(),
        Code::PartlyCloudy | Code::Overcast => partly_cloudy_icon(),
        Code::Fog(_) => foggy_icon(),
        Code::Drizzle(_)
        | Code::FreezingDrizzle(_)
        | Code::Rain(_)
        | Code::FreezingRain(_)
        | Code::RainShowers(_) => rainy_icon(),
        Code::SnowFall(_) | Code::SnowGrains | Code::SnowShowers(_) => snowy_icon(),
        Code::Thunderstorm | Code::ThunderstormWithHail(_) => thunderstorm_icon(),
        Code::Unknown => question_mark_icon(),
    }
}

pub(crate) fn weather_code_to_description(code: Code) -> String {
    match code {
        Code::ClearSky => "Clear Sky".to_string(),
        Code::MainlyClear => "Mainly Clear".to_string(),
        Code::PartlyCloudy => "Partly Cloudy".to_string(),
        Code::Overcast => "Overcast".to_string(),
        Code::Fog(fog) => format!(
            "Fog ({})",
            match fog {
                Fog::DepositingRime => "Depositing Rime",
                Fog::Normal => "Normal",
            }
        ),
        Code::Drizzle(drizle) => format!(
            "Drizzle ({})",
            match drizle {
                Drizzle::Light => "Light",
                Drizzle::Moderate => "Moderate",
                Drizzle::Dense => "Dense",
            }
        ),
        Code::FreezingDrizzle(freezing_drizzle) => format!(
            "Freezing Drizzle ({})",
            match freezing_drizzle {
                FreezingDrizzle::Light => "Light",
                FreezingDrizzle::Dense => "Dense",
            }
        ),
        Code::Rain(rain) => format!(
            "Rain ({})",
            match rain {
                Rain::Slight => "Slight",
                Rain::Moderate => "Moderate",
                Rain::Heavy => "Heavy",
            }
        ),
        Code::FreezingRain(freezing_rain) => format!(
            "Freezing Rain ({})",
            match freezing_rain {
                FreezingRain::Light => "Light",
                FreezingRain::Heavy => "Heavy",
            }
        ),
        Code::SnowFall(snow_fall) => format!(
            "Snow Fall ({})",
            match snow_fall {
                SnowFall::Slight => "Slight",
                SnowFall::Moderate => "Moderate",
                SnowFall::Heavy => "Heavy",
            }
        ),
        Code::SnowGrains => "Snow Grains".to_string(),
        Code::RainShowers(rain_showers) => format!(
            "Rain Showers ({})",
            match rain_showers {
                RainShowers::Slight => "Slight",
                RainShowers::Moderate => "Moderate",
                RainShowers::Violent => "Violent",
            }
        ),
        Code::SnowShowers(snow_showers) => format!(
            "Snow Showers ({})",
            match snow_showers {
                SnowShowers::Slight => "Slight",
                SnowShowers::Heavy => "Heavy",
            }
        ),
        Code::Thunderstorm => "Thunderstorm".to_string(),
        Code::ThunderstormWithHail(thunderstorm_with_hail) => format!(
            "Thunderstorm With Hail({})",
            match thunderstorm_with_hail {
                ThunderstormWithHail::Sight => "Sight",
                ThunderstormWithHail::Heavy => "Heavy",
            }
        ),
        Code::Unknown => "Unknown".to_string(),
    }
}
