#pragma once

#include "bindings.h"
#include "ui/include/top_bar/tray.h"

GtkWidget *tray_icon_popover_new(IO_CArray_TrayItem items, tray_triggered_f cb);
