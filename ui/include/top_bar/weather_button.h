#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherButton, weather_button, WEATHER_BUTTON, Widget,
                     GtkButton)

GtkWidget *weather_button_new();
void weather_button_refresh(WeatherButton *button, float temperature,
                            IO_WeatherCode code);

#define WEATHER_BUTTON(obj)                                                    \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), weather_button_get_type(), WeatherButton))
