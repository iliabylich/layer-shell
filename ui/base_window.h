#pragma once

#include "ui/window_model.h"
#include <gtk/gtk.h>

G_DECLARE_DERIVABLE_TYPE(BaseWindow, base_window, BASE_WINDOW, WINDOW,
                         GtkWindow)

#define BASE_WINDOW_TYPE base_window_get_type()
#define BASE_WINDOW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, base_window_get_type(), BaseWindow)

struct _BaseWindowClass {
  GtkWindowClass parent_class;
};

void base_window_vte(BaseWindow *self, char **command);
void base_window_set_window_visible(BaseWindow *self, bool visible);
void base_window_toggle_window_visible(BaseWindow *self);
