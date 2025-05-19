#include "ui/include/weather/temperature_icon.h"
#include "ui/include/weather_helper.h"

GtkWidget *temperature_icon_new() {
  GtkWidget *label = gtk_label_new("");
  gtk_widget_add_css_class(label, "icon");
  return label;
}

void temperature_icon_refresh(GtkWidget *icon, IO_WeatherCode code) {
  gtk_label_set_label(GTK_LABEL(icon), weather_code_to_icon(code));
  gtk_widget_set_tooltip_text(icon, weather_code_to_description(code));
}
