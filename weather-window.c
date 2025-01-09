#include "weather-window.h"
#include "bindings.h"
#include "utils.h"
#include "weather-helper.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define ns(name) weather_ns_##name

GtkWindow *ns(window);

typedef struct {
  GtkWidget *wrapper;
  GtkWidget *label;
  GtkWidget *image;
} weather_row_t;

#define HOURLY_ROWS_COUNT 10
weather_row_t ns(hourly_rows)[HOURLY_ROWS_COUNT];

#define DAILY_ROWS_COUNT 6
weather_row_t ns(daily_rows)[DAILY_ROWS_COUNT];

static const uint32_t ns(WIDTH) = 340;

static weather_row_t ns(make_row)(void) {
  GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  GtkWidget *label = gtk_label_new("...");
  GtkWidget *image = gtk_image_new();
  gtk_image_set_pixel_size(GTK_IMAGE(image), 24);
  gtk_box_append(GTK_BOX(row), label);
  gtk_box_append(GTK_BOX(row), image);
  return (weather_row_t){.wrapper = row, .image = image, .label = label};
}

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(ns(window)), "WeatherWindow");
  gtk_widget_add_css_class(GTK_WIDGET(ns(window)), "widget-weather");
  window_set_width_request(ns(window), ns(WIDTH));
  window_set_height_request(ns(window), 300);

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_window_set_child(ns(window), layout);

  GtkWidget *left_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(left_side, "weather-left-side");
  gtk_box_append(GTK_BOX(layout), left_side);

  gtk_box_append(GTK_BOX(left_side), gtk_label_new("Hourly"));
  for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
    weather_row_t row = ns(make_row)();
    gtk_box_append(GTK_BOX(left_side), row.wrapper);
    ns(hourly_rows)[i] = row;
  }

  GtkWidget *right_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(right_side, "weather-right-side");
  gtk_box_append(GTK_BOX(layout), right_side);

  gtk_box_append(GTK_BOX(right_side), gtk_label_new("Daily"));
  for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
    weather_row_t row = ns(make_row)();
    gtk_box_append(GTK_BOX(right_side), row.wrapper);
    ns(daily_rows)[i] = row;
  }
}

static void ns(toggle)(void) { flip_window_visibility(ns(window)); }

static void ns(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                             GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    ns(toggle)();
  }
}

static void ns(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ForecastWeather: {
    LAYER_SHELL_IO_CArray_WeatherOnDay daily = event->forecast_weather.daily;
    LAYER_SHELL_IO_CArray_WeatherOnHour hourly = event->forecast_weather.hourly;

    for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnHour weather = hourly.ptr[i];
      weather_row_t row = ns(hourly_rows)[i];

      char buffer[100];
      sprintf(buffer, "%s' %5.1f℃", weather.hour.ptr, weather.temperature);
      gtk_label_set_label(GTK_LABEL(row.label), buffer);
      gtk_widget_set_tooltip_text(row.label,
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(GTK_IMAGE(row.image),
                               weather_code_to_icon(weather.code));
    }

    for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnDay weather = daily.ptr[i];
      weather_row_t row = ns(daily_rows)[i];

      char buffer[100];
      sprintf(buffer, "%s: %5.1f℃ - %5.1f℃", weather.day.ptr,
              weather.temperature_min, weather.temperature_max);
      gtk_label_set_label(GTK_LABEL(row.label), buffer);
      gtk_widget_set_tooltip_text(row.label,
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(GTK_IMAGE(row.image),
                               weather_code_to_icon(weather.code));
    }
    break;
  }
  default:
    break;
  }
}

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(ns(window), "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(ns(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(ns(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(ns(window)), ctrl);

  layer_shell_io_subscribe(ns(on_io_event));

  gtk_window_present(ns(window));
  gtk_widget_set_visible(GTK_WIDGET(ns(window)), false);
}

static void ns(move)(uint32_t margin_left, uint32_t margin_top) {
  move_layer_window(ns(window), margin_left, margin_top);
}

uint32_t ns(width)(void) { return ns(WIDTH); }

window_t WEATHER = {.init = ns(init),
                    .activate = ns(activate),
                    .toggle = ns(toggle),
                    .move = ns(move),
                    .width = ns(width)};
