#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Network, network, NETWORK, Widget, GtkButton)

GtkWidget *network_new();
void network_refresh_wifi_status(Network *network,
                                 IO_COption_WifiStatus wifi_status);
void network_refresh_network_speed(Network *network, IO_CString upload_speed,
                                   IO_CString download_speed);
void network_refresh_network_list(Network *network, IO_CArray_Network list);

void network_emit_settings_clicked(Network *network);
void network_emit_ping_clicked(Network *network);
void network_emit_network_clicked(Network *network, const char *address);

#define NETWORK(obj)                                                           \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), network_get_type(), Network))
