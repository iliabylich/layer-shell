#include "ui/include/weather/hourly_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

#define COLS_COUNT 3
#define ROWS_COUNT 10

void hourly_grid_init(GtkWidget *self) {
  base_grid_init(self, COLS_COUNT, ROWS_COUNT);

  GtkWidget *child;
  for (size_t row = 0; row < ROWS_COUNT; row++) {
    child = gtk_label_new("??");
    gtk_grid_attach(GTK_GRID(self), child, 0, row, 1, 1);

    child = temperature_label_new();
    gtk_grid_attach(GTK_GRID(self), child, 1, row, 1, 1);

    child = temperature_icon_new();
    gtk_grid_attach(GTK_GRID(self), child, 2, row, 1, 1);
  }
}

void hourly_grid_refresh(GtkWidget *self, IO_CArray_WeatherOnHour weather) {
  GtkWidget *child;
  for (size_t row = 0; row < weather.len && row < ROWS_COUNT; row++) {
    IO_WeatherOnHour weather_on_hour = weather.ptr[row];

    child = gtk_grid_get_child_at(GTK_GRID(self), 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_hour.hour);

    child = gtk_grid_get_child_at(GTK_GRID(self), 1, row);
    temperature_label_refresh(child, weather_on_hour.temperature);

    child = gtk_grid_get_child_at(GTK_GRID(self), 2, row);
    temperature_icon_refresh(child, weather_on_hour.code);
  }
}
