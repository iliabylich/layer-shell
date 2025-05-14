#include "ui/include/weather.h"
#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/hourly_grid.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

struct _Weather {
  GtkWindow parent_instance;

  GtkWidget *hourly_grid;
  GtkWidget *daily_grid;
};

G_DEFINE_TYPE(Weather, weather, GTK_TYPE_WINDOW)

static void weather_class_init(WeatherClass *) {}

static void weather_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(window, "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
}

static void weather_init(Weather *self) {
  window_toggle_on_escape(GTK_WINDOW(self));
  weather_init_layer(GTK_WINDOW(self));

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

Weather *weather_new(GtkApplication *app) {
  return g_object_new(weather_get_type(),
                      //
                      "application", app,
                      //
                      "name", "WeatherWindow",
                      //
                      "css-classes", (const char *[]){"weather-window", NULL},
                      //
                      NULL);
}

void weather_refresh(Weather *weather, IO_Event_IO_ForecastWeather_Body data) {
  hourly_grid_refresh(HOURLY_GRID(weather->hourly_grid), data.hourly);
  daily_grid_refresh(DAILY_GRID(weather->daily_grid), data.daily);
}
