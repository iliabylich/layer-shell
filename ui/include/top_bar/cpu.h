#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *cpu_init();
void cpu_refresh(GtkWidget *cpu, IO_CpuUsageEvent event);
