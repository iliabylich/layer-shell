#include "include/utils/weather-helper.hpp"
#include "include/utils/icons.hpp"

namespace utils {

const char *
WeatherHelper::weather_code_to_description(layer_shell_io::WeatherCode code) {
  switch (code) {
  case layer_shell_io::WeatherCode::ClearSky:
    return "Clear Sky";
  case layer_shell_io::WeatherCode::MainlyClear:
    return "Mainly Clear";
  case layer_shell_io::WeatherCode::PartlyCloudy:
    return "Partly Cloudy";
  case layer_shell_io::WeatherCode::Overcast:
    return "Overcast";
  case layer_shell_io::WeatherCode::FogDepositingRime:
    return "Fog (Depositing Rime)";
  case layer_shell_io::WeatherCode::FogNormal:
    return "Fog (Normal)";
  case layer_shell_io::WeatherCode::DrizzleLight:
    return "Drizzle (Light)";
  case layer_shell_io::WeatherCode::DrizzleModerate:
    return "Drizzle (Moderate)";
  case layer_shell_io::WeatherCode::DrizzleDense:
    return "Drizzle (Dense)";
  case layer_shell_io::WeatherCode::FreezingDrizzleLight:
    return "Freezing Drizzle (Light)";
  case layer_shell_io::WeatherCode::FreezingDrizzleDense:
    return "Freezing Drizzle (Dense)";
  case layer_shell_io::WeatherCode::RainSlight:
    return "Rain (Slight)";
  case layer_shell_io::WeatherCode::RainModerate:
    return "Rain (Moderate)";
  case layer_shell_io::WeatherCode::RainHeavy:
    return "Rain (Heavy)";
  case layer_shell_io::WeatherCode::FreezingRainLight:
    return "Freezing Rain (Light)";
  case layer_shell_io::WeatherCode::FreezingRainHeavy:
    return "Freezing Rain (Heavy)";
  case layer_shell_io::WeatherCode::SnowFallSlight:
    return "Snow Fall (Slight)";
  case layer_shell_io::WeatherCode::SnowFallModerate:
    return "Snow Fall (Moderate)";
  case layer_shell_io::WeatherCode::SnowFallHeavy:
    return "Snow Fall (Heavy)";
  case layer_shell_io::WeatherCode::SnowGrains:
    return "Snow Grains";
  case layer_shell_io::WeatherCode::RainShowersSlight:
    return "Rain Showers (Slight)";
  case layer_shell_io::WeatherCode::RainShowersModerate:
    return "Rain Showers (Moderate)";
  case layer_shell_io::WeatherCode::RainShowersViolent:
    return "Rain Showers (Violent)";
  case layer_shell_io::WeatherCode::SnowShowersSlight:
    return "Snow Showers (Slight)";
  case layer_shell_io::WeatherCode::SnowShowersHeavy:
    return "Snow Showers (Heavy)";
  case layer_shell_io::WeatherCode::Thunderstorm:
    return "Thunderstorm";
  case layer_shell_io::WeatherCode::ThunderstormWithHailSight:
    return "Thunderstorm With Hail (Sight)";
  case layer_shell_io::WeatherCode::ThunderstormWithHailHeavy:
    return "Thunderstorm With Hail (Heavy)";
  case layer_shell_io::WeatherCode::Unknown:
    return "Unknown";
  }
}

Glib::RefPtr<const Gio::Icon> &
WeatherHelper::weather_code_to_icon(layer_shell_io::WeatherCode code) {
  switch (code) {
  case layer_shell_io::WeatherCode::ClearSky:
  case layer_shell_io::WeatherCode::MainlyClear:
    return Icons::sunny_icon();
  case layer_shell_io::WeatherCode::PartlyCloudy:
  case layer_shell_io::WeatherCode::Overcast:
    return Icons::partly_cloudy_icon();
  case layer_shell_io::WeatherCode::FogDepositingRime:
  case layer_shell_io::WeatherCode::FogNormal:
    return Icons::foggy_icon();
  case layer_shell_io::WeatherCode::DrizzleDense:
  case layer_shell_io::WeatherCode::DrizzleLight:
  case layer_shell_io::WeatherCode::DrizzleModerate:
  case layer_shell_io::WeatherCode::FreezingDrizzleLight:
  case layer_shell_io::WeatherCode::FreezingDrizzleDense:
  case layer_shell_io::WeatherCode::RainSlight:
  case layer_shell_io::WeatherCode::RainModerate:
  case layer_shell_io::WeatherCode::RainHeavy:
  case layer_shell_io::WeatherCode::FreezingRainLight:
  case layer_shell_io::WeatherCode::FreezingRainHeavy:
  case layer_shell_io::WeatherCode::RainShowersSlight:
  case layer_shell_io::WeatherCode::RainShowersModerate:
  case layer_shell_io::WeatherCode::RainShowersViolent:
    return Icons::rainy_icon();
  case layer_shell_io::WeatherCode::SnowFallSlight:
  case layer_shell_io::WeatherCode::SnowFallModerate:
  case layer_shell_io::WeatherCode::SnowFallHeavy:
  case layer_shell_io::WeatherCode::SnowGrains:
  case layer_shell_io::WeatherCode::SnowShowersSlight:
  case layer_shell_io::WeatherCode::SnowShowersHeavy:
    return Icons::snowy_icon();
  case layer_shell_io::WeatherCode::Thunderstorm:
  case layer_shell_io::WeatherCode::ThunderstormWithHailSight:
  case layer_shell_io::WeatherCode::ThunderstormWithHailHeavy:
    return Icons::thunderstorm_icon();
  case layer_shell_io::WeatherCode::Unknown:
    return Icons::question_mark_icon();
  }
}

} // namespace utils
