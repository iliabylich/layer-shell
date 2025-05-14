#pragma once

#include "bindings.h"
#include "ui/include/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Weather, weather, WEATHER, WINDOW, BaseWindow)

GtkWidget *weather_new(GtkApplication *app);
void weather_refresh(Weather *weather, IO_Event_IO_ForecastWeather_Body data);

#define WEATHER_TYPE weather_get_type()
#define WEATHER(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, WEATHER_TYPE, Weather)
