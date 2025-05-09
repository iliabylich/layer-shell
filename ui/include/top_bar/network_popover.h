#pragma once

#include "bindings.h"
#include "ui/include/top_bar/network.h"
#include <gtk/gtk.h>

GtkWidget *network_popover_new(Network *network);
void network_popover_refresh(GtkWidget *network_popover,
                             IO_CArray_Network list);
