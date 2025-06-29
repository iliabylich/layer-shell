#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*tray_triggered_f)(const char *uuid);

GtkWidget *tray_init(tray_triggered_f callback);
void tray_update_app(GtkWidget *tray, IO_TrayAppUpdatedEvent event);
void tray_remove_app(GtkWidget *tray, IO_TrayAppRemovedEvent event);
