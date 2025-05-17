#pragma once

#include <gtk/gtk.h>

GtkWidget *top_bar_init(GtkApplication *app);
GtkWidget *top_bar_get_widget_by_id(const char *id);
