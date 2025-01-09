#include "weather-window.h"
#include "bindings.h"
#include "utils.h"
#include "weather-helper.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

GtkWindow *weather_window;
typedef struct {
  GtkBox *wrapper;
  GtkLabel *label;
  GtkImage *image;
} weather_row_t;
#define WEATHER_HOURLY_ROWS_COUNT 10
weather_row_t weather_hourly_rows[WEATHER_HOURLY_ROWS_COUNT];
#define WEATHER_DAILY_ROWS_COUNT 6
weather_row_t weather_daily_rows[WEATHER_DAILY_ROWS_COUNT];

const uint32_t WEATHER_WINDOW_WIDTH = 340;

static weather_row_t weather_row_new(void) {
  GtkBox *row = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  GtkLabel *label = GTK_LABEL(gtk_label_new("..."));
  GtkImage *image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_pixel_size(image, 24);
  gtk_box_append(row, GTK_WIDGET(label));
  gtk_box_append(row, GTK_WIDGET(image));
  return (weather_row_t){.wrapper = row, .image = image, .label = label};
}

void init_weather_window(void) {
  weather_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(weather_window), "WeatherWindow");
  gtk_widget_add_css_class(GTK_WIDGET(weather_window), "widget-weather");

  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, WEATHER_WINDOW_WIDTH);
  g_object_set_property(G_OBJECT(weather_window), "width-request",
                        &width_request);

  GValue height_request = G_VALUE_INIT;
  g_value_init(&height_request, G_TYPE_INT);
  g_value_set_int(&height_request, 300);
  g_object_set_property(G_OBJECT(weather_window), "height-request",
                        &height_request);

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_window_set_child(weather_window, GTK_WIDGET(layout));

  GtkBox *left_side = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(left_side), "weather-left-side");
  gtk_box_append(layout, GTK_WIDGET(left_side));

  gtk_box_append(left_side, gtk_label_new("Hourly"));
  for (size_t i = 0; i < WEATHER_HOURLY_ROWS_COUNT; i++) {
    weather_row_t row = weather_row_new();
    gtk_box_append(left_side, GTK_WIDGET(row.wrapper));
    weather_hourly_rows[i] = row;
  }

  GtkBox *right_side = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(right_side), "weather-right-side");
  gtk_box_append(layout, GTK_WIDGET(right_side));

  gtk_box_append(right_side, gtk_label_new("Daily"));
  for (size_t i = 0; i < WEATHER_DAILY_ROWS_COUNT; i++) {
    weather_row_t row = weather_row_new();
    gtk_box_append(right_side, GTK_WIDGET(row.wrapper));
    weather_daily_rows[i] = row;
  }
}

void toggle_weather_window(void) { flip_window_visibility(weather_window); }

static void
on_weather_window_key_press(__attribute__((unused)) GtkEventControllerKey *self,
                            guint keyval, __attribute__((unused)) guint keycode,
                            __attribute__((unused)) GdkModifierType state,
                            __attribute__((unused)) gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_weather_window();
  }
}

static void weather_window_on_event(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ForecastWeather: {
    LAYER_SHELL_IO_CArray_WeatherOnDay daily = event->forecast_weather.daily;
    LAYER_SHELL_IO_CArray_WeatherOnHour hourly = event->forecast_weather.hourly;

    for (size_t i = 0; i < WEATHER_HOURLY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnHour weather = hourly.ptr[i];
      weather_row_t row = weather_hourly_rows[i];

      char buffer[100];
      sprintf(buffer, "%s' %5.1f℃", weather.hour.ptr, weather.temperature);
      gtk_label_set_label(row.label, buffer);
      gtk_widget_set_tooltip_text(GTK_WIDGET(row.label),
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(row.image, weather_code_to_icon(weather.code));
    }

    for (size_t i = 0; i < WEATHER_DAILY_ROWS_COUNT; i++) {
      LAYER_SHELL_IO_WeatherOnDay weather = daily.ptr[i];
      weather_row_t row = weather_daily_rows[i];

      char buffer[100];
      sprintf(buffer, "%s: %5.1f℃ - %5.1f℃", weather.day.ptr,
              weather.temperature_min, weather.temperature_max);
      gtk_label_set_label(row.label, buffer);
      gtk_widget_set_tooltip_text(GTK_WIDGET(row.label),
                                  weather_code_to_description(weather.code));

      gtk_image_set_from_gicon(row.image, weather_code_to_icon(weather.code));
    }
    break;
  }
  default:
    break;
  }
}

void activate_weather_window(GApplication *app) {
  gtk_window_set_application(weather_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(weather_window);
  gtk_layer_set_layer(weather_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(weather_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(weather_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(weather_window, "LayerShell/Weather");
  gtk_layer_set_keyboard_mode(weather_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(on_weather_window_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(weather_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  layer_shell_io_subscribe(weather_window_on_event);

  gtk_window_present(weather_window);
  gtk_widget_set_visible(GTK_WIDGET(weather_window), false);
}

void move_weather_window(uint32_t margin_left, uint32_t margin_top) {
  gtk_layer_set_margin(weather_window, GTK_LAYER_SHELL_EDGE_LEFT, margin_left);
  gtk_layer_set_margin(weather_window, GTK_LAYER_SHELL_EDGE_TOP, margin_top);
}

uint32_t weather_window_width(void) { return WEATHER_WINDOW_WIDTH; }
