#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

void network_popover_menu_add_settings(GMenu *menu);
void network_popover_menu_add_ping(GMenu *menu);
void network_popover_menu_add_network(GMenu *menu, IO_Network network);
GMenu *network_popover_menu_new(void);
