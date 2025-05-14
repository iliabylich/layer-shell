#pragma once

#include "bindings.h"
#include "ui/include/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Launcher, launcher, LAUNCHER, WINDOW, BaseWindow)

GtkWidget *launcher_new(GtkApplication *app);
void launcher_refresn(Launcher *launcher, IO_CArray_LauncherApp apps);
void launcher_toggle_and_reset(Launcher *launcher);

#define LAUNCHER_TYPE launcher_get_type()
#define LAUNCHER(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, LAUNCHER_TYPE, Launcher)
