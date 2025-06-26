#pragma once

#include "bindings.h"
#include "ui/include/top_bar/network.h"

GtkWidget *network_popover_new(network_settings_clicked_f on_settings_clicked,
                               network_ping_clicked_f on_ping_clicked,
                               network_address_clicked_f on_address_clicked);
void network_popover_refresh(GtkWidget *network_popover,
                             IO_CArray_NetworkData list);
