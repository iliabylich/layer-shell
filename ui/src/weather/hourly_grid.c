#include "ui/include/weather/hourly_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

struct _HourlyGrid {
  BaseGrid parent_instance;
};

G_DEFINE_TYPE(HourlyGrid, hourly_grid, TYPE_BASE_GRID)

static void hourly_grid_class_init(HourlyGridClass *) {}

static void hourly_grid_init(HourlyGrid *) {}

GtkWidget *hourly_grid_new() {
  HourlyGrid *self = g_object_new(hourly_grid_get_type(), "cols_count", 3,
                                  "rows_count", 10, NULL);

  for (size_t row = 0; row < self->parent_instance.rows_count; row++) {
#define ATTACH(widget, column)                                                 \
  gtk_grid_attach(GTK_GRID(self), widget, column, row, 1, 1)

    ATTACH(gtk_label_new("??"), 0);
    ATTACH(temperature_label_new(), 1);
    ATTACH(temperature_icon_new(), 2);

#undef ATTACH
  }

  return GTK_WIDGET(self);
}

static void hourly_grid_refresh_row(HourlyGrid *self,
                                    IO_WeatherOnHour weather_on_hour,
                                    size_t row) {
#define CHILD_AT(column) gtk_grid_get_child_at(GTK_GRID(self), column, row)

  gtk_label_set_text(GTK_LABEL(CHILD_AT(0)), weather_on_hour.hour);
  temperature_label_refresh(CHILD_AT(1), weather_on_hour.temperature);
  temperature_icon_refresh(CHILD_AT(2), weather_on_hour.code);

#undef CHILD_AT
}

void hourly_grid_refresh(HourlyGrid *grid, IO_CArray_WeatherOnHour data) {
  for (size_t i = 0; i < data.len && i < grid->parent_instance.rows_count;
       i++) {
    hourly_grid_refresh_row(grid, data.ptr[i], i);
  }
}
