#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *network_popover_new(GtkWidget *network);
void network_popover_refresh(GtkWidget *network_popover,
                             IO_CArray_NetworkData list);
