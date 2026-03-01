#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherDayItem, weather_day_item, WEATHER_DAY_ITEM, ITEM,
                     GObject)

#define WEATHER_DAY_ITEM(obj)                                                  \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_day_item_get_type(), WeatherDayItem)

WeatherDayItem *weather_day_item_new(IO_WeatherOnDay weather_on_day);
