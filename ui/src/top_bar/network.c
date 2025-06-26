#include "ui/include/top_bar/network.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/network_popover.h"
#include "ui/include/utils/has_prop.h"

WIDGET_HAS_PROP(status_label, GtkWidget *)
WIDGET_HAS_PROP(download_speed_label, GtkWidget *)
WIDGET_HAS_PROP(upload_speed_label, GtkWidget *)
WIDGET_HAS_PROP(popover, GtkWidget *)

static void open_popover(GtkWidget *self) {
  gtk_popover_popup(GTK_POPOVER(get_popover(self)));
}

GtkWidget *network_init(network_settings_clicked_f on_settings_clicked,
                        network_ping_clicked_f on_ping_clicked,
                        network_address_clicked_f on_address_clicked) {
  GtkWidget *self = top_bar_get_widget("NETWORK");

  GtkWidget *label = top_bar_get_widget("NETWORK_NAME");
  set_status_label(self, label);

  GtkWidget *download_speed = top_bar_get_widget("NETWORK_DOWNLOAD_SPEED");
  set_download_speed_label(self, download_speed);

  GtkWidget *upload_speed = top_bar_get_widget("NETWORK_UPLOAD_SPEED");
  set_upload_speed_label(self, upload_speed);

  GtkWidget *popover = network_popover_new(on_settings_clicked, on_ping_clicked,
                                           on_address_clicked);
  set_popover(self, popover);
  gtk_widget_set_parent(GTK_WIDGET(popover), GTK_WIDGET(self));
  g_signal_connect(self, "clicked", G_CALLBACK(open_popover), NULL);

  return self;
}

void network_refresh_wifi_status(GtkWidget *self, IO_WifiStatusEvent event) {
  GtkWidget *status_label = get_status_label(self);

  if (event.wifi_status.tag == IO_COption_WifiStatus_None_WifiStatus) {
    gtk_label_set_label(GTK_LABEL(status_label), "Not connected");
  } else {
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", event.wifi_status.some.ssid,
            event.wifi_status.some.strength);
    gtk_label_set_label(GTK_LABEL(status_label), buffer);
  }
}

void network_refresh_upload_speed(GtkWidget *self, IO_UploadSpeedEvent event) {
  gtk_label_set_label(GTK_LABEL(get_upload_speed_label(self)), event.speed);
}
void network_refresh_download_speed(GtkWidget *self,
                                    IO_DownloadSpeedEvent event) {
  gtk_label_set_label(GTK_LABEL(get_download_speed_label(self)), event.speed);
}

void network_refresh_network_list(GtkWidget *self, IO_NetworkListEvent event) {
  network_popover_refresh(get_popover(self), event.list);
}
