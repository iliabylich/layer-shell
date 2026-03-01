#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherModel, weather_model, WEATHER,
                     MODEL, GObject)

#define WEATHER_MODEL(obj)                                                     \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_model_get_type(), WeatherModel)

WeatherModel *weather_model_new(void);
