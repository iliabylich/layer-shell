#include "ui/include/top_bar/weather_button.h"
#include "ui/include/builder.h"
#include "ui/include/weather_helper.h"

GtkWidget *weather_button_init(weather_button_clicked_f callback) {
  GtkWidget *self = top_bar_get_widget("WEATHER_BUTTON");
  g_signal_connect(self, "clicked", callback, NULL);
  return self;
}

void weather_button_refresh(GtkWidget *button, float temperature,
                            IO_WeatherCode code) {
  char buffer[100];
  sprintf(buffer, "%.1f℃ %s", temperature, weather_code_to_description(code));
  gtk_button_set_label(GTK_BUTTON(button), buffer);
}
