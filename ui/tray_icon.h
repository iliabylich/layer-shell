#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrayIcon, tray_icon, TRAY_ICON, WIDGET, GtkWidget)

#define TRAY_ICON(obj)                                                         \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tray_icon_get_type(), TrayIcon)

GtkWidget *tray_icon_new(IO_TrayIcon icon, IO_FFIArray_TrayItem items);

void tray_icon_update_icon(TrayIcon *tray_icon, IO_TrayIcon icon);
void tray_icon_update_menu(TrayIcon *tray_icon, IO_FFIArray_TrayItem items);
