#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Launcher, launcher, LAUNCHER, WINDOW, GtkWindow)

Launcher *launcher_new(GtkApplication *app);
void launcher_refresn(Launcher *launcher, IO_CArray_LauncherApp apps);
void launcher_toggle_and_reset(Launcher *launcher);
