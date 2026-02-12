#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CpuItem, cpu_item, CPU, ITEM, GObject)

#define CPU_ITEM(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, cpu_item_get_type(), CpuItem)

CpuItem *cpu_item_new(void);
