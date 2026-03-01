#pragma once

#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(SessionWindow, session_window, SESSION_WINDOW, WINDOW,
                     BaseWindow)

#define SESSION_WINDOW(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, session_window_get_type(), SessionWindow)

GtkWidget *session_window_new(GtkApplication *app);
