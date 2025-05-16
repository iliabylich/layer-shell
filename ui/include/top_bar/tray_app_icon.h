#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrayAppIcon, tray_app_icon, TRAY_APP_ICON, Widget, GtkBox)

GtkWidget *tray_app_icon_new(IO_TrayApp tray_app, Tray *tray);
void tray_app_icon_cleanup(TrayAppIcon *tray_app_icon);

#define TRAY_APP_ICON_TYPE tray_app_icon_get_type()
#define TRAY_APP_ICON(obj)                                                     \
  G_TYPE_CHECK_INSTANCE_CAST(obj, TRAY_APP_ICON_TYPE, TrayAppIcon)
