#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *launcher_row_new();
void launcher_row_update(GtkWidget *row, IO_LauncherApp app);
