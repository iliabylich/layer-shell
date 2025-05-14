#pragma once

#include "bindings.h"
#include "ui/include/weather/base_grid.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(DailyGrid, daily_grid, DAILY_GRID, Widget, BaseGrid)

GtkWidget *daily_grid_new();
void daily_grid_refresh(DailyGrid *grid, IO_CArray_WeatherOnDay data);

#define DAILY_GRID_TYPE daily_grid_get_type()
#define DAILY_GRID(obj)                                                        \
  G_TYPE_CHECK_INSTANCE_CAST(obj, DAILY_GRID_TYPE, DailyGrid)
