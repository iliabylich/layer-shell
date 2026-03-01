#pragma once

#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(PingWindow, ping_window, PING_WINDOW, WINDOW, BaseWindow)

#define PING_WINDOW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, ping_window_get_type(), PingWindow)

GtkWidget *ping_window_new(GtkApplication *app);
