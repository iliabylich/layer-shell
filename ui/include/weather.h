#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *weather_init(GtkApplication *app);
void weather_refresh(GtkWidget *weather, IO_Event_IO_ForecastWeather_Body data);
void weather_toggle(GtkWidget *weather);
