#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Weather, weather, WEATHER, WINDOW, GtkWindow)

Weather *weather_new(GtkApplication *app);
void weather_refresh(Weather *weather, IO_Event_IO_ForecastWeather_Body data);
