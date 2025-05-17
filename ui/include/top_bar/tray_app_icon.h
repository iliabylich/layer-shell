#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *tray_app_icon_new(IO_TrayApp tray_app, GtkWidget *tray);
void tray_app_icon_cleanup(GtkWidget *tray_app_icon);
