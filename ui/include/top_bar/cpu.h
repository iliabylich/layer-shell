#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *cpu_init();
void cpu_refresh(GtkWidget *cpu, IO_CArray_usize usage_per_core);
