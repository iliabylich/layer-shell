#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*weather_button_clicked_f)();

GtkWidget *weather_button_init(weather_button_clicked_f callback);
void weather_button_refresh(GtkWidget *button, float temperature,
                            IO_WeatherCode code);
