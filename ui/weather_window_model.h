#pragma once

#include "ui/window_model.h"

G_DECLARE_FINAL_TYPE(WeatherWindowModel, weather_window_model, WEATHER,
                     WINDOW_MODEL, WindowModel)

#define WEATHER_WINDOW_MODEL(obj)                                              \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_window_model_get_type(),             \
                             WeatherWindowModel)

WeatherWindowModel *weather_window_model_new(void);
