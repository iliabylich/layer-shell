#include "include/utils/weather-helper.hpp"
#include "include/utils/icons.hpp"

namespace utils {

const char *WeatherHelper::weather_code_to_description(io::WeatherCode code) {
  switch (code) {
  case io::WeatherCode::ClearSky:
    return "Clear Sky";
  case io::WeatherCode::MainlyClear:
    return "Mainly Clear";
  case io::WeatherCode::PartlyCloudy:
    return "Partly Cloudy";
  case io::WeatherCode::Overcast:
    return "Overcast";
  case io::WeatherCode::FogDepositingRime:
    return "Fog (Depositing Rime)";
  case io::WeatherCode::FogNormal:
    return "Fog (Normal)";
  case io::WeatherCode::DrizzleLight:
    return "Drizzle (Light)";
  case io::WeatherCode::DrizzleModerate:
    return "Drizzle (Moderate)";
  case io::WeatherCode::DrizzleDense:
    return "Drizzle (Dense)";
  case io::WeatherCode::FreezingDrizzleLight:
    return "Freezing Drizzle (Light)";
  case io::WeatherCode::FreezingDrizzleDense:
    return "Freezing Drizzle (Dense)";
  case io::WeatherCode::RainSlight:
    return "Rain (Slight)";
  case io::WeatherCode::RainModerate:
    return "Rain (Moderate)";
  case io::WeatherCode::RainHeavy:
    return "Rain (Heavy)";
  case io::WeatherCode::FreezingRainLight:
    return "Freezing Rain (Light)";
  case io::WeatherCode::FreezingRainHeavy:
    return "Freezing Rain (Heavy)";
  case io::WeatherCode::SnowFallSlight:
    return "Snow Fall (Slight)";
  case io::WeatherCode::SnowFallModerate:
    return "Snow Fall (Moderate)";
  case io::WeatherCode::SnowFallHeavy:
    return "Snow Fall (Heavy)";
  case io::WeatherCode::SnowGrains:
    return "Snow Grains";
  case io::WeatherCode::RainShowersSlight:
    return "Rain Showers (Slight)";
  case io::WeatherCode::RainShowersModerate:
    return "Rain Showers (Moderate)";
  case io::WeatherCode::RainShowersViolent:
    return "Rain Showers (Violent)";
  case io::WeatherCode::SnowShowersSlight:
    return "Snow Showers (Slight)";
  case io::WeatherCode::SnowShowersHeavy:
    return "Snow Showers (Heavy)";
  case io::WeatherCode::Thunderstorm:
    return "Thunderstorm";
  case io::WeatherCode::ThunderstormWithHailSight:
    return "Thunderstorm With Hail (Sight)";
  case io::WeatherCode::ThunderstormWithHailHeavy:
    return "Thunderstorm With Hail (Heavy)";
  case io::WeatherCode::Unknown:
    return "Unknown";
  }
}

Glib::RefPtr<const Gio::Icon> &
WeatherHelper::weather_code_to_icon(io::WeatherCode code) {
  switch (code) {
  case io::WeatherCode::ClearSky:
  case io::WeatherCode::MainlyClear:
    return Icons::sunny;
  case io::WeatherCode::PartlyCloudy:
  case io::WeatherCode::Overcast:
    return Icons::partly_cloudy;
  case io::WeatherCode::FogDepositingRime:
  case io::WeatherCode::FogNormal:
    return Icons::foggy;
  case io::WeatherCode::DrizzleDense:
  case io::WeatherCode::DrizzleLight:
  case io::WeatherCode::DrizzleModerate:
  case io::WeatherCode::FreezingDrizzleLight:
  case io::WeatherCode::FreezingDrizzleDense:
  case io::WeatherCode::RainSlight:
  case io::WeatherCode::RainModerate:
  case io::WeatherCode::RainHeavy:
  case io::WeatherCode::FreezingRainLight:
  case io::WeatherCode::FreezingRainHeavy:
  case io::WeatherCode::RainShowersSlight:
  case io::WeatherCode::RainShowersModerate:
  case io::WeatherCode::RainShowersViolent:
    return Icons::rainy;
  case io::WeatherCode::SnowFallSlight:
  case io::WeatherCode::SnowFallModerate:
  case io::WeatherCode::SnowFallHeavy:
  case io::WeatherCode::SnowGrains:
  case io::WeatherCode::SnowShowersSlight:
  case io::WeatherCode::SnowShowersHeavy:
    return Icons::snowy;
  case io::WeatherCode::Thunderstorm:
  case io::WeatherCode::ThunderstormWithHailSight:
  case io::WeatherCode::ThunderstormWithHailHeavy:
    return Icons::thunderstorm;
  case io::WeatherCode::Unknown:
    return Icons::question_mark;
  }
}

} // namespace utils
