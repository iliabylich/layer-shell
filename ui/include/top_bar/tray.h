#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Tray, tray, TRAY, Widget, GtkBox)

#define TRAY_ACTION_ROOT_PREFIX "root"
#define TRAY_ACTION_NAMESPACE "tray"

GtkWidget *tray_new();
void tray_emit_triggered(Tray *tray, char *uuid);
void tray_refresh(Tray *tray, IO_CArray_TrayApp apps);

#define TRAY_TYPE tray_get_type()
#define TRAY(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, TRAY_TYPE, Tray)
