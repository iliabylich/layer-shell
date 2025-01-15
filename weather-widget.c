#include "weather-widget.h"
#include "bindings.h"
#include "top-bar-window.h"
#include "weather-helper.h"
#include "weather-window.h"
#include <gtk/gtk.h>

#define _(name) weather_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);

static void _(init)(void) {
  _(label) = gtk_label_new("--");
  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "weather");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_button_set_child(GTK_BUTTON(_(widget)), _(label));
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case CurrentWeather: {
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

static GtkWidget *_(main_widget)(void) { return _(widget); }

widget_t WEATHER_WIDGET = {
    .init = _(init), .activate = _(activate), .main_widget = _(main_widget)};
