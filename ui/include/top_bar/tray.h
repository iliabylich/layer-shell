#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

#define TRAY_ACTION_ROOT_PREFIX "root"
#define TRAY_ACTION_NAMESPACE "tray"

typedef void (*tray_triggered_f)(const uint8_t *uuid);

GtkWidget *tray_init(tray_triggered_f callback);
void tray_emit_triggered(GtkWidget *tray, char *uuid);
void tray_refresh(GtkWidget *tray, IO_CArray_TrayApp apps);
