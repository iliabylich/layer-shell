#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CpuLabel, cpu_label, CPU_LABEL, WIDGET, GtkWidget)

#define CPU_LABEL(obj)                                                         \
  G_TYPE_CHECK_INSTANCE_CAST(obj, cpu_label_get_type(), CpuLabel)

GtkWidget *cpu_label_new(void);
