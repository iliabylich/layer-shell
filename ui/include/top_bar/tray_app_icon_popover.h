#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *tray_app_icon_popover_new(IO_TrayItem tray_item, GtkWidget *tray,
                                     GList **context_pool);
