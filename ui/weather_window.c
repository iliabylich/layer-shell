#include "ui/weather_window.h"
#include "ui/logger.h"
#include "ui/weather_day_item.h"
#include "ui/weather_hour_item.h"

LOGGER("WeatherWindow", 0)

struct _WeatherWindow {
  GtkWidget parent_instance;

  GListStore *hourly_forecast;
  GListStore *daily_forecast;
};

G_DEFINE_TYPE(WeatherWindow, weather_window, BASE_WINDOW_TYPE)

#define HOURLY_ROWS_COUNT 10
#define DAILY_ROWS_COUNT 6

enum {
  PROP_HOURLY_FORECAST = 1,
  PROP_DAILY_FORECAST,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static char *format_temperature(GObject *, double temperature) {
  return g_strdup_printf("%5.1f℃", temperature);
}

static char *format_hour_label(GObject *, int64_t unix_seconds) {
  if (unix_seconds <= 0)
    return g_strdup("--");

  GDateTime *dt = g_date_time_new_from_unix_local(unix_seconds);
  if (!dt)
    return g_strdup("--");

  char *formatted = g_date_time_format(dt, "%H:%M");
  g_date_time_unref(dt);
  return formatted ? formatted : g_strdup("--");
}

static char *format_day_label(GObject *, int64_t unix_seconds) {
  if (unix_seconds <= 0)
    return g_strdup("--");

  GDateTime *dt = g_date_time_new_from_unix_local(unix_seconds);
  if (!dt)
    return g_strdup("--");

  char *formatted = g_date_time_format(dt, "%b-%d");
  g_date_time_unref(dt);
  return formatted ? formatted : g_strdup("--");
}

static void weather_window_get_property(GObject *object, guint property_id,
                                        GValue *value, GParamSpec *pspec) {
  WeatherWindow *self = WEATHER_WINDOW(object);

  switch (property_id) {
  case PROP_HOURLY_FORECAST:
    g_value_set_object(value, self->hourly_forecast);
    break;
  case PROP_DAILY_FORECAST:
    g_value_set_object(value, self->daily_forecast);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void weather_window_init(WeatherWindow *self) {
  LOG("init");
  self->hourly_forecast = g_list_store_new(weather_hour_item_get_type());
  self->daily_forecast = g_list_store_new(weather_day_item_get_type());
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void weather_window_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), weather_window_get_type());
  G_OBJECT_CLASS(weather_window_parent_class)->dispose(object);
}

static void weather_window_finalize(GObject *object) {
  WeatherWindow *self = WEATHER_WINDOW(object);
  g_clear_object(&self->hourly_forecast);
  g_clear_object(&self->daily_forecast);
  G_OBJECT_CLASS(weather_window_parent_class)->finalize(object);
}

static void weather_window_class_init(WeatherWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = weather_window_get_property;
  object_class->dispose = weather_window_dispose;
  object_class->finalize = weather_window_finalize;

  properties[PROP_HOURLY_FORECAST] = g_param_spec_object(
      "hourly-forecast", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_DAILY_FORECAST] = g_param_spec_object(
      "daily-forecast", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  g_type_ensure(weather_hour_item_get_type());
  g_type_ensure(weather_day_item_get_type());

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/weather_window.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_temperature);
  gtk_widget_class_bind_template_callback(widget_class, format_hour_label);
  gtk_widget_class_bind_template_callback(widget_class, format_day_label);
}

GtkWidget *weather_window_new(GtkApplication *app) {
  return g_object_new(weather_window_get_type(), "application", app, NULL);
}

void weather_window_refresh_hourly_forecast(
    WeatherWindow *self, struct IO_FFIArray_WeatherOnHour data) {
  g_list_store_remove_all(self->hourly_forecast);
  for (size_t row = 0; row < data.len && row < HOURLY_ROWS_COUNT; row++) {
    WeatherHourItem *item = weather_hour_item_new(data.ptr[row]);
    g_list_store_append(self->hourly_forecast, item);
    g_object_unref(item);
  }
}

void weather_window_refresh_daily_forecast(
    WeatherWindow *self, struct IO_FFIArray_WeatherOnDay data) {
  g_list_store_remove_all(self->daily_forecast);
  for (size_t row = 0; row < data.len && row < DAILY_ROWS_COUNT; row++) {
    WeatherDayItem *item = weather_day_item_new(data.ptr[row]);
    g_list_store_append(self->daily_forecast, item);
    g_object_unref(item);
  }
}
