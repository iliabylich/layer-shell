#include "weather.h"
#include "bindings.h"
#include "utils/weather-helper.h"
#include "windows/top-bar.h"
#include "windows/weather.h"
#include <gtk/gtk.h>

#define _(name) weather_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "weather");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_widget_set_name(_(widget), "Weather");

  _(label) = gtk_label_new("--");
  gtk_button_set_child(GTK_BUTTON(_(widget)), _(label));

  return _(widget);
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_CurrentWeather: {
    char buffer[100];
    sprintf(buffer, "%.1f℃ %s", event->current_weather.temperature,
            weather_code_to_description(event->current_weather.code));
    gtk_label_set_label(GTK_LABEL(_(label)), buffer);
    break;
  }
  default: {
    break;
  }
  }
}

static void _(on_click)(void) {
  graphene_point_t bottom_right;
  if (!bottom_right_point_of(_(widget), TOP_BAR.window(), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the weather widget");
    return;
  }
  int margin_left = bottom_right.x - WEATHER.width;
  int margin_top = bottom_right.y;
  WEATHER.move(margin_left, margin_top);

  WEATHER.toggle();
}

static void _(activate)(void) {
  g_signal_connect(_(widget), "clicked", _(on_click), NULL);
  layer_shell_io_subscribe(_(on_io_event));
}

widget_t WEATHER_WIDGET = {.init = _(init), .activate = _(activate)};
