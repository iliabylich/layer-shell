#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Weather, weather, WEATHER, WIDGET, GtkWidget)

#define WEATHER(obj)                                                           \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_get_type(), Weather)

GtkWidget *weather_new(void);
void weather_refresh(Weather *weather, IO_CurrentWeatherEvent event);
