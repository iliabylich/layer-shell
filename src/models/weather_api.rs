use chrono::{NaiveDate, NaiveDateTime};
use gtk4::glib::Priority;
use soup::{prelude::SessionExt, Message, Session};

use crate::utils::singleton;

pub(crate) struct WeatherApi {
    callbacks: Vec<Box<dyn Fn(&'static Weather) + 'static>>,
    weather: Option<Weather>,
}
singleton!(WeatherApi);

impl WeatherApi {
    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(&'static Weather) + 'static,
    {
        this().callbacks.push(Box::new(f));
    }

    pub(crate) fn get_cached() -> Option<&'static Weather> {
        this().weather.as_ref()
    }

    pub(crate) fn spawn() {
        Self::set(Self {
            callbacks: vec![],
            weather: None,
        });

        gtk4::glib::spawn_future_local(async move {
            match Self::get_weather_from_api().await {
                Ok(weather) => {
                    this().weather = Some(weather);
                    for f in this().callbacks.iter() {
                        f(Self::get_cached().unwrap());
                    }
                }
                Err(err) => log::error!("Failed to get weather data: {}", err),
            }
            gtk4::glib::timeout_future_seconds(60).await;
        });
    }

    async fn get_weather_from_api() -> Result<Weather, Box<dyn std::error::Error>> {
        let session = Session::new();
        let uri = Self::uri();
        let message = Message::builder().method("GET").uri(&uri).build();

        let bytes = session
            .send_and_read_future(&message, Priority::DEFAULT)
            .await?;
        let body = String::from_utf8(bytes.to_vec())?;
        let response = serde_json::from_str::<Response>(&body)?;

        Ok(Weather::from(response))
    }

    fn query() -> String {
        const QUERY: &[(&str, &str)] = &[
            ("latitude", "52.2298"),
            ("longitude", "21.0118"),
            ("current", "temperature_2m,weather_code"),
            ("hourly", "temperature_2m,weather_code"),
            (
                "daily",
                "temperature_2m_min,temperature_2m_max,weather_code",
            ),
            ("timezone", "Europe/Warsaw"),
        ];
        QUERY
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&")
    }

    fn uri() -> gtk4::glib::Uri {
        gtk4::glib::Uri::build(
            gtk4::glib::UriFlags::NONE,
            "https",
            None,
            Some("api.open-meteo.com"),
            443,
            "/v1/forecast",
            Some(&Self::query()),
            None,
        )
    }
}

#[derive(serde::Deserialize, Debug)]
struct Response {
    current: CurrentResponse,
    hourly: HourlyResponse,
    daily: DailyResponse,
}

#[derive(serde::Deserialize, Debug)]
struct CurrentResponse {
    temperature_2m: f32,
    weather_code: u32,
}

#[derive(serde::Deserialize, Debug)]
struct HourlyResponse {
    time: Vec<String>,
    temperature_2m: Vec<f32>,
    weather_code: Vec<u32>,
}

#[derive(serde::Deserialize, Debug)]
struct DailyResponse {
    time: Vec<String>,
    temperature_2m_min: Vec<f32>,
    temperature_2m_max: Vec<f32>,
    weather_code: Vec<u32>,
}

#[derive(Debug)]
pub(crate) struct Weather {
    pub(crate) current: CurrentWeather,
    pub(crate) hourly: Vec<HourlyWeather>,
    pub(crate) daily: Vec<DailyWeather>,
}

#[derive(Debug)]
pub(crate) struct CurrentWeather {
    pub(crate) temperature: f32,
    pub(crate) code: WeatherCode,
}

#[derive(Debug)]
pub(crate) struct HourlyWeather {
    pub(crate) hour: NaiveDateTime,
    pub(crate) temperature: f32,
    pub(crate) code: WeatherCode,
}

#[derive(Debug)]
pub(crate) struct DailyWeather {
    pub(crate) day: NaiveDate,
    pub(crate) temperature_min: f32,
    pub(crate) temperature_max: f32,
    pub(crate) code: WeatherCode,
}

impl From<Response> for Weather {
    fn from(response: Response) -> Self {
        Self {
            current: CurrentWeather::from(response.current),
            hourly: HourlyWeather::map_all(response.hourly),
            daily: DailyWeather::map_all(response.daily),
        }
    }
}

impl From<CurrentResponse> for CurrentWeather {
    fn from(response: CurrentResponse) -> Self {
        Self {
            temperature: response.temperature_2m,
            code: WeatherCode::from(response.weather_code),
        }
    }
}

impl HourlyWeather {
    fn map_all(response: HourlyResponse) -> Vec<Self> {
        let len = response.temperature_2m.len();
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let temperature = response.temperature_2m[i];
            let code = WeatherCode::from(response.weather_code[i]);
            let hour = NaiveDateTime::parse_from_str(&response.time[i], "%Y-%m-%dT%H:%M").unwrap();
            out.push(Self {
                temperature,
                hour,
                code,
            })
        }
        out
    }
}

impl DailyWeather {
    fn map_all(response: DailyResponse) -> Vec<Self> {
        let len = response.temperature_2m_max.len();
        let mut out = Vec::with_capacity(len);

        for i in 0..len {
            let temperature_min = response.temperature_2m_min[i];
            let temperature_max = response.temperature_2m_max[i];
            let code = WeatherCode::from(response.weather_code[i]);
            let day = NaiveDate::parse_from_str(&response.time[i], "%Y-%m-%d").unwrap();
            out.push(Self {
                day,
                temperature_min,
                temperature_max,
                code,
            })
        }

        out
    }
}

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

impl std::fmt::Display for WeatherCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for Fog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fog::Normal => write!(f, "Normal"),
            Fog::DepositingRime => write!(f, "Depositing Rime"),
        }
    }
}

impl std::fmt::Display for Drizzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Drizzle::Light => write!(f, "Light"),
            Drizzle::Moderate => write!(f, "Moderate"),
            Drizzle::Dense => write!(f, "Dense"),
        }
    }
}

impl std::fmt::Display for FreezingDrizzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FreezingDrizzle::Light => write!(f, "Light"),
            FreezingDrizzle::Dense => write!(f, "Dense"),
        }
    }
}

impl std::fmt::Display for Rain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rain::Slight => write!(f, "Slight"),
            Rain::Moderate => write!(f, "Moderate"),
            Rain::Heavy => write!(f, "Heavy"),
        }
    }
}

impl std::fmt::Display for FreezingRain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FreezingRain::Light => write!(f, "Light"),
            FreezingRain::Heavy => write!(f, "Heavy"),
        }
    }
}

impl std::fmt::Display for SnowFall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnowFall::Slight => write!(f, "Slight"),
            SnowFall::Moderate => write!(f, "Moderate"),
            SnowFall::Heavy => write!(f, "Heavy"),
        }
    }
}

impl std::fmt::Display for RainShowers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RainShowers::Slight => write!(f, "Slight"),
            RainShowers::Moderate => write!(f, "Moderate"),
            RainShowers::Violent => write!(f, "Violent"),
        }
    }
}

impl std::fmt::Display for SnowShowers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnowShowers::Slight => write!(f, "Slight"),
            SnowShowers::Heavy => write!(f, "Heavy"),
        }
    }
}

impl std::fmt::Display for ThunderstormWithHail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThunderstormWithHail::Sight => write!(f, "Sight"),
            ThunderstormWithHail::Heavy => write!(f, "Heavy"),
        }
    }
}
