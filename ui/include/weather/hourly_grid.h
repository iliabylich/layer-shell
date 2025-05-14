#pragma once

#include "bindings.h"
#include "ui/include/weather/base_grid.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(HourlyGrid, hourly_grid, HOURLY_GRID, Widget, BaseGrid)

GtkWidget *hourly_grid_new();
void hourly_grid_refresh(HourlyGrid *grid, IO_CArray_WeatherOnHour data);

#define HOURLY_GRID_TYPE hourly_grid_get_type()
#define HOURLY_GRID(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, HOURLY_GRID_TYPE, HourlyGrid)
