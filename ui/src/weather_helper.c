#include "ui/include/weather_helper.h"

const char *weather_code_to_description(IO_WeatherCode code) {
  switch (code) {
  case IO_WeatherCode_ClearSky:
    return "Clear Sky";
  case IO_WeatherCode_MainlyClear:
    return "Mainly Clear";
  case IO_WeatherCode_PartlyCloudy:
    return "Partly Cloudy";
  case IO_WeatherCode_Overcast:
    return "Overcast";
  case IO_WeatherCode_FogDepositingRime:
    return "Fog (Depositing Rime)";
  case IO_WeatherCode_FogNormal:
    return "Fog (Normal)";
  case IO_WeatherCode_DrizzleLight:
    return "Drizzle (Light)";
  case IO_WeatherCode_DrizzleModerate:
    return "Drizzle (Moderate)";
  case IO_WeatherCode_DrizzleDense:
    return "Drizzle (Dense)";
  case IO_WeatherCode_FreezingDrizzleLight:
    return "Freezing Drizzle (Light)";
  case IO_WeatherCode_FreezingDrizzleDense:
    return "Freezing Drizzle (Dense)";
  case IO_WeatherCode_RainSlight:
    return "Rain (Slight)";
  case IO_WeatherCode_RainModerate:
    return "Rain (Moderate)";
  case IO_WeatherCode_RainHeavy:
    return "Rain (Heavy)";
  case IO_WeatherCode_FreezingRainLight:
    return "Freezing Rain (Light)";
  case IO_WeatherCode_FreezingRainHeavy:
    return "Freezing Rain (Heavy)";
  case IO_WeatherCode_SnowFallSlight:
    return "Snow Fall (Slight)";
  case IO_WeatherCode_SnowFallModerate:
    return "Snow Fall (Moderate)";
  case IO_WeatherCode_SnowFallHeavy:
    return "Snow Fall (Heavy)";
  case IO_WeatherCode_SnowGrains:
    return "Snow Grains";
  case IO_WeatherCode_RainShowersSlight:
    return "Rain Showers (Slight)";
  case IO_WeatherCode_RainShowersModerate:
    return "Rain Showers (Moderate)";
  case IO_WeatherCode_RainShowersViolent:
    return "Rain Showers (Violent)";
  case IO_WeatherCode_SnowShowersSlight:
    return "Snow Showers (Slight)";
  case IO_WeatherCode_SnowShowersHeavy:
    return "Snow Showers (Heavy)";
  case IO_WeatherCode_Thunderstorm:
    return "Thunderstorm";
  case IO_WeatherCode_ThunderstormWithHailSight:
    return "Thunderstorm With Hail (Sight)";
  case IO_WeatherCode_ThunderstormWithHailHeavy:
    return "Thunderstorm With Hail (Heavy)";
  case IO_WeatherCode_Unknown:
    return "Unknown";
  }

  return NULL;
}

const char *weather_code_to_icon(IO_WeatherCode code) {
  switch (code) {
  case IO_WeatherCode_ClearSky:
  case IO_WeatherCode_MainlyClear:
    return "󰖙";
  case IO_WeatherCode_PartlyCloudy:
  case IO_WeatherCode_Overcast:
    return "󰖐";
  case IO_WeatherCode_FogDepositingRime:
  case IO_WeatherCode_FogNormal:
    return "󰖑";
  case IO_WeatherCode_DrizzleDense:
  case IO_WeatherCode_DrizzleLight:
  case IO_WeatherCode_DrizzleModerate:
  case IO_WeatherCode_FreezingDrizzleLight:
  case IO_WeatherCode_FreezingDrizzleDense:
  case IO_WeatherCode_RainSlight:
  case IO_WeatherCode_RainModerate:
  case IO_WeatherCode_RainHeavy:
  case IO_WeatherCode_FreezingRainLight:
  case IO_WeatherCode_FreezingRainHeavy:
  case IO_WeatherCode_RainShowersSlight:
  case IO_WeatherCode_RainShowersModerate:
  case IO_WeatherCode_RainShowersViolent:
    return "󰖗";
  case IO_WeatherCode_SnowFallSlight:
  case IO_WeatherCode_SnowFallModerate:
  case IO_WeatherCode_SnowFallHeavy:
  case IO_WeatherCode_SnowGrains:
  case IO_WeatherCode_SnowShowersSlight:
  case IO_WeatherCode_SnowShowersHeavy:
    return "󰖘";
  case IO_WeatherCode_Thunderstorm:
  case IO_WeatherCode_ThunderstormWithHailSight:
  case IO_WeatherCode_ThunderstormWithHailHeavy:
    return "";
  case IO_WeatherCode_Unknown:
    return "";
  }

  return "";
}
