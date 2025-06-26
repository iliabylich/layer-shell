#pragma once

#include "ui/include/top_bar/network.h"

GSimpleActionGroup *
network_popover_action_map_new(network_settings_clicked_f on_settings_clicked,
                               network_ping_clicked_f on_ping_clicked,
                               network_address_clicked_f on_address_clicked);
