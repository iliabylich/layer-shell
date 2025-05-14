#include "ui/include/top_bar/weather_button.h"
#include "gtk/gtk.h"
#include "ui/include/weather_helper.h"

struct _WeatherButton {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(WeatherButton, weather_button, GTK_TYPE_BUTTON)

static void weather_button_class_init(WeatherButtonClass *) {}

static const char *css_classes[] = {"widget", "weather", "padded", "clickable",
                                    NULL};

static void weather_button_init(WeatherButton *self) {
  gtk_button_set_label(GTK_BUTTON(self), "--");
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "WeatherButton");
}

GtkWidget *weather_button_new() {
  return g_object_new(weather_button_get_type(), NULL);
}

void weather_button_refresh(WeatherButton *button, float temperature,
                            IO_WeatherCode code) {
  char buffer[100];
  sprintf(buffer, "%.1f℃ %s", temperature, weather_code_to_description(code));
  gtk_button_set_label(GTK_BUTTON(button), buffer);
}
