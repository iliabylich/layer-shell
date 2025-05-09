#include "ui/include/weather/temperature_icon.h"
#include "ui/include/icons.h"
#include "ui/include/weather_helper.h"

GtkWidget *temperature_icon_new() {
  return gtk_image_new_from_gicon(get_question_mark_icon());
}

void temperature_icon_refresh(GtkWidget *icon, IO_WeatherCode code) {
  gtk_image_set_from_gicon(GTK_IMAGE(icon), weather_code_to_icon(code));
  gtk_widget_set_tooltip_text(icon, weather_code_to_description(code));
}
