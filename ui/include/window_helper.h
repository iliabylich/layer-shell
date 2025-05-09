#pragma once

#include <gtk/gtk.h>

void window_toggle(GtkWindow *window);
void window_toggle_on_escape(GtkWindow *window);
void vte_window(GtkWindow *window, char **command);
