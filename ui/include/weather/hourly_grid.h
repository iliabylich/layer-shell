#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

void hourly_grid_init(GtkWidget *grid);
void hourly_grid_refresh(GtkWidget *grid, IO_CArray_WeatherOnHour data);
