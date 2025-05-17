#pragma once

#include <gtk/gtk.h>

typedef void (*change_theme_clicked_f)();

GtkWidget *change_theme_init(change_theme_clicked_f callback);
