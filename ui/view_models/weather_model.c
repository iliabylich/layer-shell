#include "ui/view_models/weather_model.h"
#include "ui/view_models/weather_day_item.h"
#include "ui/view_models/weather_hour_item.h"
#include "ui/weather_helper.h"

struct _WeatherModel {
  GObject parent_instance;

  char *text;
  GListStore *hourly_forecast;
  GListStore *daily_forecast;
};

G_DEFINE_TYPE(WeatherModel, weather_model, G_TYPE_OBJECT)

enum {
  PROP_TEXT = 1,
  PROP_HOURLY_FORECAST,
  PROP_DAILY_FORECAST,
  PROP_DATA,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void weather_model_set_weather(WeatherModel *self,
                                      struct IO_Event_IO_Weather_Body weather) {
  char buffer[100];
  snprintf(buffer, sizeof(buffer), "%.1f\xe2\x84\x83 %s", weather.temperature,
           weather_code_to_description(weather.code));
  g_free(self->text);
  self->text = g_strdup(buffer);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_TEXT]);

  g_list_store_remove_all(self->hourly_forecast);
  for (size_t i = 0; i < weather.hourly_forecast.len; i++) {
    WeatherHourItem *item = weather_hour_item_new(weather.hourly_forecast.ptr[i]);
    g_list_store_append(self->hourly_forecast, item);
    g_object_unref(item);
  }

  g_list_store_remove_all(self->daily_forecast);
  for (size_t i = 0; i < weather.daily_forecast.len; i++) {
    WeatherDayItem *item = weather_day_item_new(weather.daily_forecast.ptr[i]);
    g_list_store_append(self->daily_forecast, item);
    g_object_unref(item);
  }
}

static void weather_model_get_property(GObject *object, guint property_id,
                                       GValue *value, GParamSpec *pspec) {
  WeatherModel *self = WEATHER_MODEL(object);
  switch (property_id) {
  case PROP_TEXT:
    g_value_set_string(value, self->text);
    break;
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

static void weather_model_set_property(GObject *object, guint property_id,
                                       const GValue *value,
                                       GParamSpec *pspec) {
  WeatherModel *self = WEATHER_MODEL(object);
  switch (property_id) {
  case PROP_DATA: {
    IO_Event_IO_Weather_Body *data = g_value_get_pointer(value);
    if (data)
      weather_model_set_weather(self, *data);
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void weather_model_finalize(GObject *object) {
  WeatherModel *self = WEATHER_MODEL(object);
  g_free(self->text);
  g_clear_object(&self->hourly_forecast);
  g_clear_object(&self->daily_forecast);
  G_OBJECT_CLASS(weather_model_parent_class)->finalize(object);
}

static void weather_model_init(WeatherModel *self) {
  self->text = g_strdup("--");
  self->hourly_forecast = g_list_store_new(weather_hour_item_get_type());
  self->daily_forecast = g_list_store_new(weather_day_item_get_type());
}

static void weather_model_class_init(WeatherModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = weather_model_get_property;
  object_class->set_property = weather_model_set_property;
  object_class->finalize = weather_model_finalize;

  properties[PROP_TEXT] =
      g_param_spec_string("text", NULL, NULL, "--", G_PARAM_READABLE);
  properties[PROP_HOURLY_FORECAST] =
      g_param_spec_object("hourly-forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_DAILY_FORECAST] =
      g_param_spec_object("daily-forecast", NULL, NULL,
                          G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  properties[PROP_DATA] =
      g_param_spec_pointer("data", NULL, NULL, G_PARAM_WRITABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WeatherModel *weather_model_new(void) {
  return g_object_new(weather_model_get_type(), NULL);
}
