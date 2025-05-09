#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"
#include <gio/gio.h>

GMenu *tray_menu_new(IO_TrayItem tray_item, GActionGroup *action_group,
                     Tray *tray);
