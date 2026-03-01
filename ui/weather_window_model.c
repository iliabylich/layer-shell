#include "ui/weather_window_model.h"

struct _WeatherWindowModel {
  WindowModel parent_instance;
};

G_DEFINE_TYPE(WeatherWindowModel, weather_window_model, window_model_get_type())

static void weather_window_model_init(WeatherWindowModel *) {}
static void weather_window_model_class_init(WeatherWindowModelClass *) {}

WeatherWindowModel *weather_window_model_new(void) {
  return g_object_new(weather_window_model_get_type(), NULL);
}
