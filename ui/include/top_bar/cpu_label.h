#pragma once

#include <gtk/gtk.h>

#define CpuLabel GtkLabel
#define CPU_LABEL GTK_LABEL

GtkWidget *cpu_label_new();
void cpu_label_set_load(CpuLabel *label, float load);
