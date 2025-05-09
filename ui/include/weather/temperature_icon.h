#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *temperature_icon_new();
void temperature_icon_refresh(GtkWidget *icon, IO_WeatherCode code);
