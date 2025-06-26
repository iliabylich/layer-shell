#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *weather_init(GtkApplication *app);
void weather_refresh_hourly_forecast(GtkWidget *weather,
                                     IO_HourlyWeatherForecastEvent event);
void weather_refresh_daily_forecast(GtkWidget *weather,
                                    IO_DailyWeatherForecastEvent event);
void weather_toggle(GtkWidget *weather);
