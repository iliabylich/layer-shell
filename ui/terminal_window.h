#pragma once

#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TerminalWindow, terminal_window, TERMINAL_WINDOW, WINDOW,
                     BaseWindow)

#define TERMINAL_WINDOW(obj)                                                   \
  G_TYPE_CHECK_INSTANCE_CAST(obj, terminal_window_get_type(), TerminalWindow)

GtkWidget *terminal_window_new(GtkApplication *app);
void terminal_window_toggle(TerminalWindow *terminal_window);
