#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CpuModel, cpu_model, CPU, MODEL, GObject)

#define CPU_MODEL(obj)                                                         \
  G_TYPE_CHECK_INSTANCE_CAST(obj, cpu_model_get_type(), CpuModel)

CpuModel *cpu_model_new(void);
