#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GMenu *tray_icon_popover_menu_new(IO_CArray_TrayItem items);
