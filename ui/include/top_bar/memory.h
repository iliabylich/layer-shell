#pragma once

#include <gtk/gtk.h>

typedef void (*memory_clicked_f)();

GtkWidget *memory_init(memory_clicked_f callback);
void memory_refresh(GtkWidget *memory, double used, double total);
