#include "ui/include/weather.h"
#include "ui/include/builder.h"
#include "ui/include/utils/has_prop.h"
#include "ui/include/weather/daily_grid.h"
#include "ui/include/weather/hourly_grid.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

WIDGET_HAS_PROP(hourly_grid, GtkWidget *)
WIDGET_HAS_PROP(daily_grid, GtkWidget *)

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
  set_hourly_grid(self, hourly_grid);

  GtkWidget *daily_grid = weather_get_widget("DAILY");
  daily_grid_init(daily_grid);
  set_daily_grid(self, daily_grid);

  return self;
}

void weather_refresh_hourly_forecast(GtkWidget *self,
                                     IO_HourlyWeatherForecastEvent event) {

  hourly_grid_refresh(get_hourly_grid(self), event.forecast);
}
void weather_refresh_daily_forecast(GtkWidget *self,
                                    IO_DailyWeatherForecastEvent event) {
  daily_grid_refresh(get_daily_grid(self), event.forecast);
}

void weather_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }
