#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherHourItem, weather_hour_item, WEATHER_HOUR_ITEM,
                     ITEM, GObject)

#define WEATHER_HOUR_ITEM(obj)                                                 \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_hour_item_get_type(), WeatherHourItem)

WeatherHourItem *weather_hour_item_new(IO_WeatherOnHour weather_on_hour);
