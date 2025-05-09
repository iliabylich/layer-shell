#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Cpu, cpu, CPU, Widget, GtkBox)

GtkWidget *cpu_new();
void cpu_refresh(Cpu *cpu, IO_CArray_usize usage_per_core);

#define CPU(obj) (G_TYPE_CHECK_INSTANCE_CAST((obj), cpu_get_type(), Cpu))
