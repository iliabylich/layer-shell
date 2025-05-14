#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

struct _DailyGrid {
  BaseGrid parent_instance;
};

G_DEFINE_TYPE(DailyGrid, daily_grid, BASE_GRID_TYPE)

static void daily_grid_class_init(DailyGridClass *) {}

static void daily_grid_init(DailyGrid *) {}

GtkWidget *daily_grid_new() {
  DailyGrid *self = g_object_new(DAILY_GRID_TYPE,
                                 //
                                 "cols_count", 4,
                                 //
                                 "rows_count", 6,
                                 //
                                 NULL);

  for (size_t row = 0; row < self->parent_instance.rows_count; row++) {
#define ATTACH(widget, column)                                                 \
  gtk_grid_attach(GTK_GRID(self), widget, column, row, 1, 1);

    ATTACH(gtk_label_new("??"), 0);
    ATTACH(temperature_label_new(), 1);
    ATTACH(temperature_label_new(), 2);
    ATTACH(temperature_icon_new(), 3);

#undef ATTACH
  }

  return GTK_WIDGET(self);
}

static void daily_grid_refresh_row(DailyGrid *self,
                                   IO_WeatherOnDay weather_on_day, size_t row) {
#define CHILD_AT(column) gtk_grid_get_child_at(GTK_GRID(self), column, row)

  gtk_label_set_text(GTK_LABEL(CHILD_AT(0)), weather_on_day.day);
  temperature_label_refresh(CHILD_AT(1), weather_on_day.temperature_min);
  temperature_label_refresh(CHILD_AT(2), weather_on_day.temperature_max);
  temperature_icon_refresh(CHILD_AT(3), weather_on_day.code);

#undef CHILD_AT
}

void daily_grid_refresh(DailyGrid *grid, IO_CArray_WeatherOnDay data) {
  for (size_t i = 0; i < data.len && i < grid->parent_instance.rows_count;
       i++) {
    daily_grid_refresh_row(grid, data.ptr[i], i);
  }
}
