#pragma once

#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(HTopWindow, htop_window, HTOP_WINDOW, WINDOW, BaseWindow)

#define HTOP_WINDOW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, htop_window_get_type(), HTopWindow)

GtkWidget *htop_window_new(GtkApplication *app);
void htop_window_toggle(HTopWindow *htop_window);
