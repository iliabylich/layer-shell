#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(ClockModel, clock_model, CLOCK, MODEL, GObject)

#define CLOCK_MODEL(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, clock_model_get_type(), ClockModel)

ClockModel *clock_model_new(void);
