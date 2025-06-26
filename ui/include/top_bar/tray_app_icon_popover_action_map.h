#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"

GActionGroup *tray_app_icon_popover_action_map_new(IO_TrayItem tray_item,
                                                   tray_triggered_f cb);
