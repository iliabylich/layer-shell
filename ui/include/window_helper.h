#pragma once

#include <gtk/gtk.h>

void window_toggle(GtkWindow *window);
void window_set_toggle_on_escape(GtkWindow *window);
void window_vte(GtkWindow *window, char **command);
