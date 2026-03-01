#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(MemoryModel, memory_model, MEMORY, MODEL, GObject)

#define MEMORY_MODEL(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, memory_model_get_type(), MemoryModel)

MemoryModel *memory_model_new(void);
