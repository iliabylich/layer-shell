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

typedef void (*network_settings_clicked_f)();
typedef void (*network_ping_clicked_f)();
typedef void (*network_address_clicked_f)(const char *address);

GtkWidget *network_init(network_settings_clicked_f on_settings_clicked,
                        network_ping_clicked_f on_ping_clicked,
                        network_address_clicked_f on_address_clicked);

void network_refresh_wifi_status(GtkWidget *network,
                                 IO_COption_WifiStatus wifi_status);
void network_refresh_network_speed(GtkWidget *network, const char *upload_speed,
                                   const char *download_speed);
void network_refresh_network_list(GtkWidget *network, IO_CArray_Network list);

void network_emit_settings_clicked(GtkWidget *network);
void network_emit_ping_clicked(GtkWidget *network);
void network_emit_network_clicked(GtkWidget *network, const char *address);
