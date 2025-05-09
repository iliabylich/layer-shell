#pragma once

#include <gtk/gtk.h>

GtkWidget *temperature_label_new();
void temperature_label_refresh(GtkWidget *label, float temperature);
