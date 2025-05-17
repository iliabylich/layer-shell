#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GActionGroup *tray_app_icon_popover_action_map_new(IO_TrayItem tray_item,
                                                   GtkWidget *tray,
                                                   GList **context_pool);
