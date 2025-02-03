#include "weather.h"
#include "bindings.h"
#include "utils/weather-helper.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) weather_ns_##name

static GtkWindow *_(window);

typedef struct {
  GtkWidget *wrapper;
  GtkWidget *label;
  GtkWidget *image;
} row_t;

#define HOURLY_ROWS_COUNT 10
static row_t _(hourly_rows)[HOURLY_ROWS_COUNT];

#define DAILY_ROWS_COUNT 6
static row_t _(daily_rows)[DAILY_ROWS_COUNT];

static const int _(WIDTH) = 340;

static row_t _(make_row)(void) {
  GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  GtkWidget *label = gtk_label_new("...");
  GtkWidget *image = gtk_image_new();
  gtk_image_set_pixel_size(GTK_IMAGE(image), 24);
  gtk_box_append(GTK_BOX(row), label);
  gtk_box_append(GTK_BOX(row), image);
  return (row_t){.wrapper = row, .image = image, .label = label};
}

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "WeatherWindow");
  gtk_widget_add_css_class(GTK_WIDGET(_(window)), "widget-weather");
  window_set_width_request(_(window), _(WIDTH));
  window_set_height_request(_(window), 300);

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_window_set_child(_(window), layout);

  GtkWidget *left_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(left_side, "weather-left-side");
  gtk_box_append(GTK_BOX(layout), left_side);

  gtk_box_append(GTK_BOX(left_side), gtk_label_new("Hourly"));
  for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
    row_t row = _(make_row)();
    gtk_box_append(GTK_BOX(left_side), row.wrapper);
    _(hourly_rows)[i] = row;
  }

  GtkWidget *right_side = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(right_side, "weather-right-side");
  gtk_box_append(GTK_BOX(layout), right_side);

  gtk_box_append(GTK_BOX(right_side), gtk_label_new("Daily"));
  for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
    row_t row = _(make_row)();
    gtk_box_append(GTK_BOX(right_side), row.wrapper);
    _(daily_rows)[i] = row;
  }
}

static void _(toggle)(void) { flip_window_visibility(_(window)); }

static void _(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                            GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    _(toggle)();
  }
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_ForecastWeather: {
    IO_CArray_WeatherOnDay daily = event->forecast_weather.daily;
    IO_CArray_WeatherOnHour hourly = event->forecast_weather.hourly;

    for (size_t i = 0; i < HOURLY_ROWS_COUNT; i++) {
      IO_WeatherOnHour weather = hourly.ptr[i];
      row_t row = _(hourly_rows)[i];

      char buffer[100];
      sprintf(buffer, "%s' %5.1f℃", weather.hour, weather.temperature);
      gtk_label_set_label(GTK_LABEL(row.label), buffer);
      gtk_widget_set_tooltip_text(row.label,
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(GTK_IMAGE(row.image),
                               weather_code_to_icon(weather.code));
    }

    for (size_t i = 0; i < DAILY_ROWS_COUNT; i++) {
      IO_WeatherOnDay weather = daily.ptr[i];
      row_t row = _(daily_rows)[i];

      char buffer[100];
      sprintf(buffer, "%s: %5.1f℃ - %5.1f℃", weather.day,
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

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(_(window), "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(_(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(_(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(_(window)), ctrl);

  layer_shell_io_subscribe(_(on_io_event));

  gtk_window_present(_(window));
  gtk_widget_set_visible(GTK_WIDGET(_(window)), false);
}

static void _(move)(int x, int y) { move_layer_window(_(window), x, y); }

window_t WEATHER = {.init = _(init),
                    .activate = _(activate),
                    .toggle = _(toggle),
                    .move = _(move),
                    .width = _(WIDTH)};
