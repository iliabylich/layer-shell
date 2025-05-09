#pragma once

#include <gtk/gtk.h>

GtkLabel *cpu_label_new();
void cpu_label_set_load(GtkLabel *label, float load);
