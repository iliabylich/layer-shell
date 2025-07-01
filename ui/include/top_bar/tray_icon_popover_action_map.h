#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"

GActionGroup *tray_icon_popover_action_map_new(IO_CArray_TrayItem items,
                                               tray_triggered_f cb);
