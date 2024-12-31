#include "bindings.h"
#include "gio/gio.h"
#include "icons.h"

const char *weather_code_to_description(LAYER_SHELL_IO_WeatherCode code) {
  switch (code.tag) {
  case ClearSky:
    return "Clear Sky";
  case MainlyClear:
    return "Mainly Clear";
  case PartlyCloudy:
    return "Partly Cloudy";
  case Overcast:
    return "Overcast";
  case Fog: {
    switch (code.fog) {
    case FogDepositingRime:
      return "Fog (Depositing Rime)";
    case FogNormal:
      return "Fog (Normal)";
    }
    break;
  }
  case Drizzle: {
    switch (code.drizzle) {
    case DrizzleLight:
      return "Drizzle (Light)";
    case DrizzleModerate:
      return "Drizzle (Moderate)";
    case DrizzleDense:
      return "Drizzle (Dense)";
    }
    break;
  }
  case FreezingDrizzle: {
    switch (code.freezing_drizzle) {
    case FreezingDrizzleLight:
      return "Freezing Drizzle (Light)";
    case FreezingDrizzleDense:
      return "Freezing Drizzle (Dense)";
    }
    break;
  }
  case Rain: {
    switch (code.rain) {
    case RainSlight:
      return "Rain (Slight)";
    case RainModerate:
      return "Rain (Moderate)";
    case RainHeavy:
      return "Rain (Heavy)";
    }
    break;
  }
  case FreezingRain: {
    switch (code.freezing_rain) {
    case FreezingRainLight:
      return "Freezing Rain (Light)";
    case FreezingRainHeavy:
      return "Freezing Rain (Heavy)";
    }
    break;
  }
  case SnowFall: {
    switch (code.snow_fall) {
    case SnowFallSlight:
      return "Snow Fall (Slight)";
    case SnowFallModerate:
      return "Snow Fall (Moderate)";
    case SnowFallHeavy:
      return "Snow Fall (Heavy)";
    }
    break;
  }
  case SnowGrains:
    return "Snow Grains";
  case RainShowers: {
    switch (code.rain_showers) {
    case RainShowersSlight:
      return "Rain Showers (Slight)";
    case RainShowersModerate:
      return "Rain Showers (Moderate)";
    case RainShowersViolent:
      return "Rain Showers (Violent)";
    }
    break;
  }
  case SnowShowers: {
    switch (code.snow_showers) {
    case SnowShowersSlight:
      return "Snow Showers (Slight)";
    case SnowShowersHeavy:
      return "Snow Showers (Heavy)";
    }
    break;
  }
  case Thunderstorm:
    return "Thunderstorm";
  case ThunderstormWithHail: {
    switch (code.thunderstorm_with_hail) {
    case ThunderstormWithHailSight:
      return "Thunderstorm With Hail (Sight)";
    case ThunderstormWithHailHeavy:
      return "Thunderstorm With Hail (Heavy)";
    }
    break;
  }
  case Unknown:
    return "Unknown";
  }

  return NULL;
}

GIcon *weather_code_to_icon(LAYER_SHELL_IO_WeatherCode code) {
  switch (code.tag) {
  case ClearSky:
  case MainlyClear:
    return get_icon(SUNNY_ICON);
  case PartlyCloudy:
  case Overcast:
    return get_icon(PARTLY_CLOUDY_ICON);
  case Fog:
    return get_icon(FOGGY_ICON);
  case Drizzle:
  case FreezingDrizzle:
  case Rain:
  case FreezingRain:
  case RainShowers:
    return get_icon(RAINY_ICON);
  case SnowFall:
  case SnowGrains:
  case SnowShowers:
    return get_icon(SNOWY_ICON);
  case Thunderstorm:
  case ThunderstormWithHail:
    return get_icon(THUNDERSTORM_ICON);
  case Unknown:
    return get_icon(QUESTION_MARK_ICON);
  }

  return NULL;
}
