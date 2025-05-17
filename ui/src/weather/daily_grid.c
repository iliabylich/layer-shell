#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

#define COLS_COUNT 4
#define ROWS_COUNT 6

void daily_grid_init(GtkWidget *self) {
  base_grid_init(self, COLS_COUNT, ROWS_COUNT);

  base_grid_data_t *data = base_grid_get_data(self);
  for (size_t row = 0; row < data->rows_count; row++) {
#define ATTACH(widget, column)                                                 \
  gtk_grid_attach(GTK_GRID(self), widget, column, row, 1, 1)

    ATTACH(gtk_label_new("??"), 0);
    ATTACH(temperature_label_new(), 1);
    ATTACH(temperature_label_new(), 2);
    ATTACH(temperature_icon_new(), 3);

#undef ATTACH
  }
}

static void daily_grid_refresh_row(GtkWidget *self,
                                   IO_WeatherOnDay weather_on_day, size_t row) {
#define CHILD_AT(column) gtk_grid_get_child_at(GTK_GRID(self), column, row)

  gtk_label_set_text(GTK_LABEL(CHILD_AT(0)), weather_on_day.day);
  temperature_label_refresh(CHILD_AT(1), weather_on_day.temperature_min);
  temperature_label_refresh(CHILD_AT(2), weather_on_day.temperature_max);
  temperature_icon_refresh(CHILD_AT(3), weather_on_day.code);

#undef CHILD_AT
}

void daily_grid_refresh(GtkWidget *self, IO_CArray_WeatherOnDay weather) {
  base_grid_data_t *data = base_grid_get_data(self);

  for (size_t i = 0; i < weather.len && i < data->rows_count; i++) {
    daily_grid_refresh_row(self, weather.ptr[i], i);
  }
}
