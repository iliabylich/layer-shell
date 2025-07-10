#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Network, network, NETWORK, WIDGET, GtkWidget)

#define NETWORK(obj)                                                           \
  G_TYPE_CHECK_INSTANCE_CAST(obj, network_get_type(), Network)

GtkWidget *network_new(void);

void network_refresh_network_ssid(Network *network, IO_NetworkSsidEvent event);
void network_refresh_network_strength(Network *network,
                                      IO_NetworkStrengthEvent event);
void network_refresh_upload_speed(Network *network, IO_UploadSpeedEvent event);
void network_refresh_download_speed(Network *network,
                                    IO_DownloadSpeedEvent event);
void network_refresh_network_list(Network *network, IO_NetworkListEvent event);
