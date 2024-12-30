#ifndef WEATHER_H
#define WEATHER_H

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
  }
  case FreezingDrizzle: {
    switch (code.freezing_drizzle) {
    case FreezingDrizzleLight:
      return "Freezing Drizzle (Light)";
    case FreezingDrizzleDense:
      return "Freezing Drizzle (Dense)";
    }
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
  }
  case FreezingRain: {
    switch (code.freezing_rain) {
    case FreezingRainLight:
      return "Freezing Rain (Light)";
    case FreezingRainHeavy:
      return "Freezing Rain (Heavy)";
    }
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
  }
  case SnowShowers: {
    switch (code.snow_showers) {
    case SnowShowersSlight:
      return "Snow Showers (Slight)";
    case SnowShowersHeavy:
      return "Snow Showers (Heavy)";
    }
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
  }
  case Unknown:
    return "Unknown";
  }
}

GIcon *weather_code_to_icon(LAYER_SHELL_IO_WeatherCode code) {
  switch (code.tag) {
  case ClearSky:
  case MainlyClear:
    return SUNNY_ICON;
  case PartlyCloudy:
  case Overcast:
    return PARTLY_CLOUDY_ICON;
  case Fog:
    return FOGGY_ICON;
  case Drizzle:
  case FreezingDrizzle:
  case Rain:
  case FreezingRain:
  case RainShowers:
    return RAINY_ICON;
  case SnowFall:
  case SnowGrains:
  case SnowShowers:
    return SNOWY_ICON;
  case Thunderstorm:
  case ThunderstormWithHail:
    return THUNDERSTORM_ICON;
  case Unknown:
    return QUESTION_MARK_ICON;
  }
}

#endif // WEATHER_H
