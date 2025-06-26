#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *tray_app_icon_new(const char *service, IO_TrayItem item,
                             IO_TrayIcon icon, GtkWidget *tray);
const char *tray_app_icon_service(GtkWidget *tray_app_icon);
void tray_app_icon_cleanup(GtkWidget *tray_app_icon);
