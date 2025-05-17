#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

void daily_grid_init(GtkWidget *grid);
void daily_grid_refresh(GtkWidget *grid, IO_CArray_WeatherOnDay data);
