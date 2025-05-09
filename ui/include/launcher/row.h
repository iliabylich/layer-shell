#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(LauncherRow, launcher_row, LAUNCHER_ROW, WIDGET, GtkBox)

GtkWidget *launcher_row_new();
void launcher_row_update(LauncherRow *row, IO_LauncherApp app);

#define LAUNCHER_ROW(obj)                                                      \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), launcher_row_get_type(), LauncherRow))
