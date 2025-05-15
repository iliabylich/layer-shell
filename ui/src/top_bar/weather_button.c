#include "ui/include/top_bar/weather_button.h"
#include "ui/include/macros.h"
#include "ui/include/weather_helper.h"

struct _WeatherButton {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(WeatherButton, weather_button, GTK_TYPE_BUTTON)

static void weather_button_class_init(WeatherButtonClass *) {}

static void weather_button_init(WeatherButton *) {}

GtkWidget *weather_button_new() {
  // clang-format off
  return g_object_new(
      WEATHER_BUTTON_TYPE,
      "label", "--",
      "css-classes", CSS("widget", "weather", "padded", "clickable"),
      "cursor", gdk_cursor_new_from_name("pointer", NULL),
      "name", "WeatherButton",
      NULL);
  // clang-format on
}

void weather_button_refresh(WeatherButton *button, float temperature,
                            IO_WeatherCode code) {
  char buffer[100];
  sprintf(buffer, "%.1f℃ %s", temperature, weather_code_to_description(code));
  gtk_button_set_label(GTK_BUTTON(button), buffer);
}
