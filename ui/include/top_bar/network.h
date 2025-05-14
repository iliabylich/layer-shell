#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

#define NETWORK_NAMESPACE "network"

#define NETWORK_SETTINGS_LABEL "Settings (nmtui)"
#define NETWORK_SETTINGS_ACTION "settings-clicked"
#define NETWORK_SETTINGS_DETAILED_ACTION "network.settings-clicked"

#define NETWORK_PING_LABEL "Ping"
#define NETWORK_PING_ACTION "ping-clicked"
#define NETWORK_PING_DETAILED_ACTION "network.ping-clicked"

#define NETWORK_ROW_ACTION "network-row-clicked"
#define NETWORK_ROW_DETAILED_ACTION "network.network-row-clicked"

G_DECLARE_FINAL_TYPE(Network, network, NETWORK, Widget, GtkButton)

GtkWidget *network_new();
void network_refresh_wifi_status(Network *network,
                                 IO_COption_WifiStatus wifi_status);
void network_refresh_network_speed(Network *network, const char *upload_speed,
                                   const char *download_speed);
void network_refresh_network_list(Network *network, IO_CArray_Network list);

void network_emit_settings_clicked(Network *network);
void network_emit_ping_clicked(Network *network);
void network_emit_network_clicked(Network *network, const char *address);

#define NETWORK(obj)                                                           \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), network_get_type(), Network))
