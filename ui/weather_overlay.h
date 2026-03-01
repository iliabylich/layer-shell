#pragma once

#include "bindings.h"
#include "ui/base_overlay.h"
#include "ui/view_models/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(WeatherOverlay, weather_overlay, WEATHER_OVERLAY, OVERLAY,
                     BaseOverlay)

#define WEATHER_OVERLAY(obj)                                                   \
  G_TYPE_CHECK_INSTANCE_CAST(obj, weather_overlay_get_type(), WeatherOverlay)

GtkWidget *weather_overlay_new(GtkApplication *app, IOModel *model);
