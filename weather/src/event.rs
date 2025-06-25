use crate::WeatherCode;
use ffi::{CArray, CString};

#[derive(Debug)]
pub enum WeatherEvent {
    CurrentWeather(CurrentWeatherEvent),
    HourlyWeatherForecast(HourlyWeatherForecastEvent),
    DailyWeatherForecast(DailyWeatherForecastEvent),
}

#[derive(Debug)]
#[repr(C)]
pub struct CurrentWeatherEvent {
    pub temperature: f32,
    pub code: WeatherCode,
}
#[derive(Debug)]
#[repr(C)]
pub struct HourlyWeatherForecastEvent {
    pub forecast: CArray<WeatherOnHour>,
}
#[derive(Debug)]
#[repr(C)]
pub struct DailyWeatherForecastEvent {
    pub forecast: CArray<WeatherOnDay>,
}

#[derive(Debug)]
#[repr(C)]
pub struct WeatherOnHour {
    pub hour: CString,
    pub temperature: f32,
    pub code: WeatherCode,
}

#[derive(Debug)]
#[repr(C)]
pub struct WeatherOnDay {
    pub day: CString,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}
