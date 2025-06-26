#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/base_grid.h"
#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather/temperature_label.h"

#define COLS_COUNT 4
#define ROWS_COUNT 6

void daily_grid_init(GtkWidget *self) {
  base_grid_init(self, COLS_COUNT, ROWS_COUNT);

  GtkWidget *child;
  for (size_t row = 0; row < ROWS_COUNT; row++) {
    child = gtk_label_new("??");
    gtk_grid_attach(GTK_GRID(self), child, 0, row, 1, 1);

    child = temperature_label_new();
    gtk_grid_attach(GTK_GRID(self), child, 1, row, 1, 1);

    child = temperature_label_new();
    gtk_grid_attach(GTK_GRID(self), child, 2, row, 1, 1);

    child = temperature_icon_new();
    gtk_grid_attach(GTK_GRID(self), child, 3, row, 1, 1);
  }
}

void daily_grid_refresh(GtkWidget *self, IO_CArray_WeatherOnDay weather) {
  GtkWidget *child;

  for (size_t row = 0; row < weather.len && row < ROWS_COUNT; row++) {
    IO_WeatherOnDay weather_on_day = weather.ptr[row];

    child = gtk_grid_get_child_at(GTK_GRID(self), 0, row);
    gtk_label_set_text(GTK_LABEL(child), weather_on_day.day);

    child = gtk_grid_get_child_at(GTK_GRID(self), 1, row);
    temperature_label_refresh(child, weather_on_day.temperature_min);

    child = gtk_grid_get_child_at(GTK_GRID(self), 2, row);
    temperature_label_refresh(child, weather_on_day.temperature_max);

    child = gtk_grid_get_child_at(GTK_GRID(self), 3, row);
    temperature_icon_refresh(child, weather_on_day.code);
  }
}
