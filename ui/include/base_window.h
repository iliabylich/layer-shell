#pragma once

#include <gtk/gtk.h>

struct _BaseWindow {
  GtkWindow parent_instance;
};

G_DECLARE_FINAL_TYPE(BaseWindow, base_window, BASE_WINDOW, WINDOW, GtkWindow)

#define BASE_WINDOW_TYPE base_window_get_type()
#define BASE_WINDOW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, BASE_WINDOW_TYPE, BaseWindow)

void window_toggle(GtkWindow *window);
