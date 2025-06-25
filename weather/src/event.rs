use crate::WeatherCode;

#[derive(Debug)]
pub enum Event {
    CurrentWeather { temperature: f32, code: WeatherCode },
    HourlyWeatherForecast { forecast: Vec<WeatherOnHour> },
    DailyWeatherForecast { forecast: Vec<WeatherOnDay> },
}

#[derive(Debug)]
pub struct WeatherOnHour {
    pub hour: String,
    pub temperature: f32,
    pub code: WeatherCode,
}

#[derive(Debug)]
pub struct WeatherOnDay {
    pub day: String,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}
