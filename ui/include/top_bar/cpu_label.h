#pragma once

#include <gtk/gtk.h>

GtkWidget *cpu_label_new();
void cpu_label_set_load(GtkWidget *label, float load);
