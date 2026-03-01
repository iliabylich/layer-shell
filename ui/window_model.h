#pragma once

#include <gtk/gtk.h>

G_DECLARE_DERIVABLE_TYPE(WindowModel, window_model, WINDOW, MODEL, GObject)

#define WINDOW_MODEL(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, window_model_get_type(), WindowModel)

struct _WindowModelClass {
  GObjectClass parent_class;
};

WindowModel *window_model_new(void);
