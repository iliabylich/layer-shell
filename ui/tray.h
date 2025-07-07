#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Tray, tray, TRAY, WIDGET, GtkWidget)

#define TRAY(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, tray_get_type(), Tray)

GtkWidget *tray_new(void);

void tray_add_app(Tray *self, IO_TrayAppAddedEvent event);
void tray_remove_app(Tray *self, IO_TrayAppRemovedEvent event);
void tray_update_icon(Tray *self, IO_TrayAppIconUpdatedEvent event);
void tray_update_menu(Tray *self, IO_TrayAppMenuUpdatedEvent event);
