#include "ui/include/weather.h"
#include "ui/include/macros.h"
#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/hourly_grid.h"
#include <gtk4-layer-shell.h>

struct _Weather {
  BaseWindow parent_instance;

  GtkWidget *hourly_grid;
  GtkWidget *daily_grid;
};

G_DEFINE_TYPE(Weather, weather, BASE_WINDOW_TYPE)

static void weather_class_init(WeatherClass *) {}

static void weather_init(Weather *self) {
  self->hourly_grid = hourly_grid_new();
  self->daily_grid = daily_grid_new();

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 50);
  gtk_window_set_child(GTK_WINDOW(self), layout);

  GtkWidget *left_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_box_append(GTK_BOX(layout), left_side);
  gtk_box_append(GTK_BOX(left_side), gtk_label_new("Hourly"));
  gtk_box_append(GTK_BOX(left_side), self->hourly_grid);

  GtkWidget *right_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_box_append(GTK_BOX(layout), right_side);
  gtk_box_append(GTK_BOX(right_side), gtk_label_new("Daily"));
  gtk_box_append(GTK_BOX(right_side), self->daily_grid);
}

GtkWidget *weather_new(GtkApplication *app) {
  // clang-format off
  return g_object_new(
      WEATHER_TYPE,
      "application", app,
      "name", "WeatherWindow",
      "css-classes", CSS("weather-window"),
      "toggle-on-escape", true,
      "layer", GTK_LAYER_SHELL_LAYER_OVERLAY,
      "layer-namespace", "LayerShell/Weather",
      "layer-keyboard-mode", GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE,
      NULL);
  // clang-format on
}

void weather_refresh(Weather *weather, IO_Event_IO_ForecastWeather_Body data) {
  hourly_grid_refresh(HOURLY_GRID(weather->hourly_grid), data.hourly);
  daily_grid_refresh(DAILY_GRID(weather->daily_grid), data.daily);
}
