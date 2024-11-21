use crate::icons::{
    foggy_icon, partly_cloudy_icon, question_mark_icon, rainy_icon, sunny_icon, thunderstorm_icon,
    weather_snowy_icon,
};
use layer_shell_io::weather::*;

pub(crate) fn weather_code_to_icon(code: WeatherCode) -> &'static gtk4::gdk::Texture {
    match code {
        WeatherCode::ClearSky | WeatherCode::MainlyClear => sunny_icon(),
        WeatherCode::PartlyCloudy | WeatherCode::Overcast => partly_cloudy_icon(),
        WeatherCode::Fog(_) => foggy_icon(),
        WeatherCode::Drizzle(_)
        | WeatherCode::FreezingDrizzle(_)
        | WeatherCode::Rain(_)
        | WeatherCode::FreezingRain(_)
        | WeatherCode::RainShowers(_) => rainy_icon(),
        WeatherCode::SnowFall(_) | WeatherCode::SnowGrains | WeatherCode::SnowShowers(_) => {
            weather_snowy_icon()
        }
        WeatherCode::Thunderstorm | WeatherCode::ThunderstormWithHail(_) => thunderstorm_icon(),
        WeatherCode::Unknown => question_mark_icon(),
    }
}

pub(crate) fn weather_code_to_description(code: WeatherCode) -> String {
    match code {
        WeatherCode::ClearSky => "Clear Sky".to_string(),
        WeatherCode::MainlyClear => "Mainly Clear".to_string(),
        WeatherCode::PartlyCloudy => "Partly Cloudy".to_string(),
        WeatherCode::Overcast => "Overcast".to_string(),
        WeatherCode::Fog(fog) => format!(
            "Fog ({})",
            match fog {
                Fog::DepositingRime => "Depositing Rime",
                Fog::Normal => "Normal",
            }
        ),
        WeatherCode::Drizzle(drizle) => format!(
            "Drizzle ({})",
            match drizle {
                Drizzle::Light => "Light",
                Drizzle::Moderate => "Moderate",
                Drizzle::Dense => "Dense",
            }
        ),
        WeatherCode::FreezingDrizzle(freezing_drizzle) => format!(
            "Freezing Drizzle ({})",
            match freezing_drizzle {
                FreezingDrizzle::Light => "Light",
                FreezingDrizzle::Dense => "Dense",
            }
        ),
        WeatherCode::Rain(rain) => format!(
            "Rain ({})",
            match rain {
                Rain::Slight => "Slight",
                Rain::Moderate => "Moderate",
                Rain::Heavy => "Heavy",
            }
        ),
        WeatherCode::FreezingRain(freezing_rain) => format!(
            "Freezing Rain ({})",
            match freezing_rain {
                FreezingRain::Light => "Light",
                FreezingRain::Heavy => "Heavy",
            }
        ),
        WeatherCode::SnowFall(snow_fall) => format!(
            "Snow Fall ({})",
            match snow_fall {
                SnowFall::Slight => "Slight",
                SnowFall::Moderate => "Moderate",
                SnowFall::Heavy => "Heavy",
            }
        ),
        WeatherCode::SnowGrains => "Snow Grains".to_string(),
        WeatherCode::RainShowers(rain_showers) => format!(
            "Rain Showers ({})",
            match rain_showers {
                RainShowers::Slight => "Slight",
                RainShowers::Moderate => "Moderate",
                RainShowers::Violent => "Violent",
            }
        ),
        WeatherCode::SnowShowers(snow_showers) => format!(
            "Snow Showers ({})",
            match snow_showers {
                SnowShowers::Slight => "Slight",
                SnowShowers::Heavy => "Heavy",
            }
        ),
        WeatherCode::Thunderstorm => "Thunderstorm".to_string(),
        WeatherCode::ThunderstormWithHail(thunderstorm_with_hail) => format!(
            "Thunderstorm With Hail({})",
            match thunderstorm_with_hail {
                ThunderstormWithHail::Sight => "Sight",
                ThunderstormWithHail::Heavy => "Heavy",
            }
        ),
        WeatherCode::Unknown => "Unknown".to_string(),
    }
}
