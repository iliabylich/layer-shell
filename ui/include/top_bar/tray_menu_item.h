#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"
#include <gio/gio.h>

GMenuItem *tray_menu_item_new(IO_TrayItem tray_item, GActionGroup *action_group,
                              size_t idx, Tray *tray);
