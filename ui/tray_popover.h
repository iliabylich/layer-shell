#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrayPopover, tray_popover, TRAY_POPOVER, WIDGET, GtkWidget)

#define TRAY_POPOVER(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tray_popover_get_type(), TrayPopover)

GtkWidget *tray_popover_new();

void tray_popover_open(TrayPopover *tray_popover);
void tray_popover_update(TrayPopover *tray_popover, IO_CArray_TrayItem items);
