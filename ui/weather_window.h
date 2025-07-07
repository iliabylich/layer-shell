#pragma once

#include "bindings.h"
#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherWindow, weather_window, WEATHER_WINDOW, WINDOW,
                     BaseWindow)

#define WEATHER_WINDOW(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_window_get_type(), WeatherWindow)

GtkWidget *weather_window_new(GtkApplication *app);
void weather_window_toggle(WeatherWindow *weather_window);
void weather_window_refresh_hourly_forecast(
    WeatherWindow *weather_window, IO_HourlyWeatherForecastEvent event);
void weather_window_refresh_daily_forecast(WeatherWindow *weather_window,
                                           IO_DailyWeatherForecastEvent event);
