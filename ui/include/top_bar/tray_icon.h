#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"

GtkWidget *tray_icon_new(IO_TrayIcon icon, IO_CArray_TrayItem items,
                         tray_triggered_f cb);
void tray_icon_cleanup(GtkWidget *tray_icon);
