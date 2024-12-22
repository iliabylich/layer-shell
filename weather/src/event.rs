use crate::Code;

pub enum WeatherEvent {
    CurrentWeather(CurrentWeather),
    ForecastWeather(ForecastWeather),
}

#[derive(Debug)]
pub struct CurrentWeather {
    pub temperature: f32,
    pub code: Code,
}

#[derive(Debug)]
pub struct ForecastWeather {
    pub hourly: Vec<WeatherOnHour>,
    pub daily: Vec<WeatherOnDay>,
}

#[derive(Debug)]
pub struct WeatherOnHour {
    pub hour: String,
    pub temperature: f32,
    pub code: Code,
}

#[derive(Debug)]
pub struct WeatherOnDay {
    pub day: String,
    pub temperature: std::ops::Range<f32>,
    pub code: Code,
}
