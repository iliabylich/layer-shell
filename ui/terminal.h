#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Terminal, terminal, TERMINAL, WIDGET, GtkWidget)

#define TERMINAL(obj)                                                          \
  G_TYPE_CHECK_INSTANCE_CAST(obj, terminal_get_type(), Terminal)

GtkWidget *terminal_new(void);
