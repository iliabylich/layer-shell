#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(BaseWindow, base_window, BASE_WINDOW, WINDOW, GtkWindow)

#define BASE_WINDOW_TYPE base_window_get_type()
#define BASE_WINDOW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, base_window_get_type(), BaseWindow)

GtkWidget *base_window_new(GtkApplication *app);

void base_window_toggle(BaseWindow *base_window);
void base_window_set_toggle_on_escape(BaseWindow *self);
void base_window_vte(BaseWindow *self, char **command);
