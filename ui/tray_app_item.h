#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrayAppItem, tray_app_item, TRAY_APP, ITEM, GObject)

#define TRAY_APP_ITEM(obj)                                                     \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tray_app_item_get_type(), TrayAppItem)

TrayAppItem *tray_app_item_new(const char *service, IO_TrayIcon icon,
                               IO_FFIArray_TrayItem items);

const char *tray_app_item_get_service(TrayAppItem *self);
void tray_app_item_update_icon(TrayAppItem *self, IO_TrayIcon icon);
void tray_app_item_update_menu(TrayAppItem *self, IO_FFIArray_TrayItem items);
