#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*network_settings_clicked_f)();
typedef void (*network_ping_clicked_f)();
typedef void (*network_address_clicked_f)(const char *address);

GtkWidget *network_init(network_settings_clicked_f on_settings_clicked,
                        network_ping_clicked_f on_ping_clicked,
                        network_address_clicked_f on_address_clicked);

void network_refresh_wifi_status(GtkWidget *network, IO_WifiStatusEvent event);
void network_refresh_upload_speed(GtkWidget *network,
                                  IO_UploadSpeedEvent event);
void network_refresh_download_speed(GtkWidget *network,
                                    IO_DownloadSpeedEvent event);
void network_refresh_network_list(GtkWidget *network,
                                  IO_NetworkListEvent event);
