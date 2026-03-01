#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(OverlaysModel, overlays_model, OVERLAYS, MODEL, GObject)

#define OVERLAYS_MODEL(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, overlays_model_get_type(), OverlaysModel)

OverlaysModel *overlays_model_new(void);
