#include "weather-helper.h"
#include "icons.h"

const char *weather_code_to_description(LAYER_SHELL_IO_WeatherCode code) {
  switch (code) {
  case ClearSky:
    return "Clear Sky";
  case MainlyClear:
    return "Mainly Clear";
  case PartlyCloudy:
    return "Partly Cloudy";
  case Overcast:
    return "Overcast";
  case FogDepositingRime:
    return "Fog (Depositing Rime)";
  case FogNormal:
    return "Fog (Normal)";
  case DrizzleLight:
    return "Drizzle (Light)";
  case DrizzleModerate:
    return "Drizzle (Moderate)";
  case DrizzleDense:
    return "Drizzle (Dense)";
  case FreezingDrizzleLight:
    return "Freezing Drizzle (Light)";
  case FreezingDrizzleDense:
    return "Freezing Drizzle (Dense)";
  case RainSlight:
    return "Rain (Slight)";
  case RainModerate:
    return "Rain (Moderate)";
  case RainHeavy:
    return "Rain (Heavy)";
  case FreezingRainLight:
    return "Freezing Rain (Light)";
  case FreezingRainHeavy:
    return "Freezing Rain (Heavy)";
  case SnowFallSlight:
    return "Snow Fall (Slight)";
  case SnowFallModerate:
    return "Snow Fall (Moderate)";
  case SnowFallHeavy:
    return "Snow Fall (Heavy)";
  case SnowGrains:
    return "Snow Grains";
  case RainShowersSlight:
    return "Rain Showers (Slight)";
  case RainShowersModerate:
    return "Rain Showers (Moderate)";
  case RainShowersViolent:
    return "Rain Showers (Violent)";
  case SnowShowersSlight:
    return "Snow Showers (Slight)";
  case SnowShowersHeavy:
    return "Snow Showers (Heavy)";
  case Thunderstorm:
    return "Thunderstorm";
  case ThunderstormWithHailSight:
    return "Thunderstorm With Hail (Sight)";
  case ThunderstormWithHailHeavy:
    return "Thunderstorm With Hail (Heavy)";
  case Unknown:
    return "Unknown";
  }

  return NULL;
}

GIcon *weather_code_to_icon(LAYER_SHELL_IO_WeatherCode code) {
  switch (code) {
  case ClearSky:
  case MainlyClear:
    return get_sunny_icon();
  case PartlyCloudy:
  case Overcast:
    return get_partly_cloudy_icon();
  case FogDepositingRime:
  case FogNormal:
    return get_foggy_icon();
  case DrizzleDense:
  case DrizzleLight:
  case DrizzleModerate:
  case FreezingDrizzleLight:
  case FreezingDrizzleDense:
  case RainSlight:
  case RainModerate:
  case RainHeavy:
  case FreezingRainLight:
  case FreezingRainHeavy:
  case RainShowersSlight:
  case RainShowersModerate:
  case RainShowersViolent:
    return get_rainy_icon();
  case SnowFallSlight:
  case SnowFallModerate:
  case SnowFallHeavy:
  case SnowGrains:
  case SnowShowersSlight:
  case SnowShowersHeavy:
    return get_snowy_icon();
  case Thunderstorm:
  case ThunderstormWithHailSight:
  case ThunderstormWithHailHeavy:
    return get_thunderstorm_icon();
  case Unknown:
    return get_question_mark_icon();
  }

  return NULL;
}
