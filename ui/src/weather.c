#include "ui/include/weather.h"
#include "gtk/gtk.h"
#include "ui/include/builder.h"
#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/hourly_grid.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

typedef struct {
  GtkWidget *hourly_grid;
  GtkWidget *daily_grid;
} data_t;
#define DATA_KEY "data"

GtkWidget *weather_init(GtkApplication *app) {
  GtkWidget *self = weather_get_widget("WEATHER");
  gtk_window_set_application(GTK_WINDOW(self), app);
  window_set_toggle_on_escape(GTK_WINDOW(self));
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkWidget *hourly_grid = weather_get_widget("HOURLY");
  hourly_grid_init(hourly_grid);
  GtkWidget *daily_grid = weather_get_widget("DAILY");
  daily_grid_init(daily_grid);

  data_t *data = malloc(sizeof(data_t));
  data->hourly_grid = hourly_grid;
  data->daily_grid = daily_grid;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  return self;
}

void weather_refresh(GtkWidget *self,
                     IO_Event_IO_ForecastWeather_Body weather) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  hourly_grid_refresh(data->hourly_grid, weather.hourly);
  daily_grid_refresh(data->daily_grid, weather.daily);
}

void weather_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }
