#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(NetworkModel, network_model, NETWORK, MODEL, GObject)

#define NETWORK_MODEL(obj)                                                     \
  G_TYPE_CHECK_INSTANCE_CAST(obj, network_model_get_type(), NetworkModel)

NetworkModel *network_model_new(void);
