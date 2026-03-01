#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrayModel, tray_model, TRAY, MODEL, GObject)

#define TRAY_MODEL(obj)                                                        \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tray_model_get_type(), TrayModel)

TrayModel *tray_model_new(void);

void tray_model_add_app(TrayModel *self, const char *service, IO_TrayIcon icon,
                        IO_FFIArray_TrayItem items);
void tray_model_remove_app(TrayModel *self, const char *service);
void tray_model_update_icon(TrayModel *self, const char *service,
                            IO_TrayIcon icon);
void tray_model_update_menu(TrayModel *self, const char *service,
                            IO_FFIArray_TrayItem items);
