#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"

GtkWidget *tray_app_icon_new(IO_TrayIcon icon, IO_TrayItem item,
                             tray_triggered_f cb);
void tray_app_icon_cleanup(GtkWidget *tray_app_icon);
