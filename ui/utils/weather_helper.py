from liblayer_shell_io import WeatherCode
from utils.context import ctx


class WeatherHelper:
    @staticmethod
    def code_to_description(code: WeatherCode):
        match code:
            case WeatherCode.ClearSky:
                return "Clear Sky"
            case WeatherCode.MainlyClear:
                return "Mainly Clear"
            case WeatherCode.PartlyCloudy:
                return "Partly Cloudy"
            case WeatherCode.Overcast:
                return "Overcast"
            case WeatherCode.FogDepositingRime:
                return "Fog (Depositing Rime)"
            case WeatherCode.FogNormal:
                return "Fog (Normal)"
            case WeatherCode.DrizzleLight:
                return "Drizzle (Light)"
            case WeatherCode.DrizzleModerate:
                return "Drizzle (Moderate)"
            case WeatherCode.DrizzleDense:
                return "Drizzle (Dense)"
            case WeatherCode.FreezingDrizzleLight:
                return "Freezing Drizzle (Light)"
            case WeatherCode.FreezingDrizzleDense:
                return "Freezing Drizzle (Dense)"
            case WeatherCode.RainSlight:
                return "Rain (Slight)"
            case WeatherCode.RainModerate:
                return "Rain (Moderate)"
            case WeatherCode.RainHeavy:
                return "Rain (Heavy)"
            case WeatherCode.FreezingRainLight:
                return "Freezing Rain (Light)"
            case WeatherCode.FreezingRainHeavy:
                return "Freezing Rain (Heavy)"
            case WeatherCode.SnowFallSlight:
                return "Snow Fall (Slight)"
            case WeatherCode.SnowFallModerate:
                return "Snow Fall (Moderate)"
            case WeatherCode.SnowFallHeavy:
                return "Snow Fall (Heavy)"
            case WeatherCode.SnowGrains:
                return "Snow Grains"
            case WeatherCode.RainShowersSlight:
                return "Rain Showers (Slight)"
            case WeatherCode.RainShowersModerate:
                return "Rain Showers (Moderate)"
            case WeatherCode.RainShowersViolent:
                return "Rain Showers (Violent)"
            case WeatherCode.SnowShowersSlight:
                return "Snow Showers (Slight)"
            case WeatherCode.SnowShowersHeavy:
                return "Snow Showers (Heavy)"
            case WeatherCode.Thunderstorm:
                return "Thunderstorm"
            case WeatherCode.ThunderstormWithHailSight:
                return "Thunderstorm With Hail (Sight)"
            case WeatherCode.ThunderstormWithHailHeavy:
                return "Thunderstorm With Hail (Heavy)"
            case WeatherCode.Unknown:
                return "Unknown"
            case _:
                return "Unsupported (bug?)"

    @staticmethod
    def code_to_icon(code: WeatherCode):
        match code:
            case WeatherCode.ClearSky | WeatherCode.MainlyClear:
                return ctx.icons.sunny
            case WeatherCode.PartlyCloudy | WeatherCode.Overcast:
                return ctx.icons.partly_cloudy
            case WeatherCode.FogDepositingRime | WeatherCode.FogNormal:
                return ctx.icons.foggy
            case (
                WeatherCode.DrizzleDense
                | WeatherCode.DrizzleLight
                | WeatherCode.DrizzleModerate
                | WeatherCode.FreezingDrizzleLight
                | WeatherCode.FreezingDrizzleDense
                | WeatherCode.RainSlight
                | WeatherCode.RainModerate
                | WeatherCode.RainHeavy
                | WeatherCode.FreezingRainLight
                | WeatherCode.FreezingRainHeavy
                | WeatherCode.RainShowersSlight
                | WeatherCode.RainShowersModerate
                | WeatherCode.RainShowersViolent
            ):
                return ctx.icons.rainy
            case (
                WeatherCode.SnowFallSlight
                | WeatherCode.SnowFallModerate
                | WeatherCode.SnowFallHeavy
                | WeatherCode.SnowGrains
                | WeatherCode.SnowShowersSlight
                | WeatherCode.SnowShowersHeavy
            ):
                return ctx.icons.snowy
            case (
                WeatherCode.Thunderstorm
                | WeatherCode.ThunderstormWithHailSight
                | WeatherCode.ThunderstormWithHailHeavy
            ):
                return ctx.icons.thunderstorm
            case WeatherCode.Unknown:
                return ctx.icons.question_mark
