#include "bindings.h"
#include "icons.h"
#include <gio/gio.h>

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
    return get_icon(SUNNY_ICON);
  case PartlyCloudy:
  case Overcast:
    return get_icon(PARTLY_CLOUDY_ICON);
  case FogDepositingRime:
  case FogNormal:
    return get_icon(FOGGY_ICON);
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
    return get_icon(RAINY_ICON);
  case SnowFallSlight:
  case SnowFallModerate:
  case SnowFallHeavy:
  case SnowGrains:
  case SnowShowersSlight:
  case SnowShowersHeavy:
    return get_icon(SNOWY_ICON);
  case Thunderstorm:
  case ThunderstormWithHailSight:
  case ThunderstormWithHailHeavy:
    return get_icon(THUNDERSTORM_ICON);
  case Unknown:
    return get_icon(QUESTION_MARK_ICON);
  }

  return NULL;
}
