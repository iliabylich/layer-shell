#include "ui/include/weather/hourly_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

#define COLS_COUNT 3
#define ROWS_COUNT 10

void hourly_grid_init(GtkWidget *self) {
  base_grid_init(self, COLS_COUNT, ROWS_COUNT);

  base_grid_data_t *data = base_grid_get_data(self);
  for (size_t row = 0; row < data->rows_count; row++) {
#define ATTACH(widget, column)                                                 \
  gtk_grid_attach(GTK_GRID(self), widget, column, row, 1, 1)

    ATTACH(gtk_label_new("??"), 0);
    ATTACH(temperature_label_new(), 1);
    ATTACH(temperature_icon_new(), 2);

#undef ATTACH
  }
}

static void hourly_grid_refresh_row(GtkWidget *self,
                                    IO_WeatherOnHour weather_on_hour,
                                    size_t row) {
#define CHILD_AT(column) gtk_grid_get_child_at(GTK_GRID(self), column, row)

  gtk_label_set_text(GTK_LABEL(CHILD_AT(0)), weather_on_hour.hour);
  temperature_label_refresh(CHILD_AT(1), weather_on_hour.temperature);
  temperature_icon_refresh(CHILD_AT(2), weather_on_hour.code);

#undef CHILD_AT
}

void hourly_grid_refresh(GtkWidget *self, IO_CArray_WeatherOnHour weather) {
  base_grid_data_t *data = base_grid_get_data(self);

  for (size_t i = 0; i < weather.len && i < data->rows_count; i++) {
    hourly_grid_refresh_row(self, weather.ptr[i], i);
  }
}
