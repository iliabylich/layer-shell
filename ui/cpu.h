#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Cpu, cpu, CPU, WIDGET, GtkWidget)

#define CPU(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, cpu_get_type(), Cpu)

GtkWidget *cpu_new(void);
void cpu_refresh(Cpu *cpu, IO_CArray_u8 data);
