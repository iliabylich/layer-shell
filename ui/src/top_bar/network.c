#include "ui/include/top_bar/network.h"
#include "gtk/gtk.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/network_popover.h"

typedef struct {
  GtkWidget *label;
  GtkWidget *download_speed;
  GtkWidget *upload_speed;
  GtkWidget *popover;

  network_settings_clicked_f on_settings_clicked;
  network_ping_clicked_f on_ping_clicked;
  network_address_clicked_f on_address_clicked;
} data_t;
#define DATA_KEY "data"

static void on_click(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  gtk_popover_popup(GTK_POPOVER(data->popover));
}

GtkWidget *network_init(network_settings_clicked_f on_settings_clicked,
                        network_ping_clicked_f on_ping_clicked,
                        network_address_clicked_f on_address_clicked) {
  GtkWidget *self = top_bar_get_widget("NETWORK");
  GtkWidget *label = top_bar_get_widget("NETWORK_NAME");
  GtkWidget *download_speed = top_bar_get_widget("NETWORK_DOWNLOAD_SPEED");
  GtkWidget *upload_speed = top_bar_get_widget("NETWORK_UPLOAD_SPEED");
  GtkWidget *popover = network_popover_new(self);

  data_t *data = malloc(sizeof(data_t));
  data->label = label;
  data->download_speed = download_speed;
  data->upload_speed = upload_speed;
  data->popover = popover;
  data->on_settings_clicked = on_settings_clicked;
  data->on_ping_clicked = on_ping_clicked;
  data->on_address_clicked = on_address_clicked;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  gtk_widget_set_parent(GTK_WIDGET(popover), GTK_WIDGET(self));
  g_signal_connect(self, "clicked", G_CALLBACK(on_click), NULL);

  return self;
}

void network_refresh_wifi_status(GtkWidget *self, IO_WifiStatusEvent event) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  if (event.wifi_status.tag == IO_COption_WifiStatus_None_WifiStatus) {
    gtk_label_set_label(GTK_LABEL(data->label), "Not connected");
  } else {
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", event.wifi_status.some.ssid,
            event.wifi_status.some.strength);
    gtk_label_set_label(GTK_LABEL(data->label), buffer);
  }
}

void network_refresh_upload_speed(GtkWidget *self, IO_UploadSpeedEvent event) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  gtk_label_set_label(GTK_LABEL(data->upload_speed), event.speed);
}
void network_refresh_download_speed(GtkWidget *self,
                                    IO_DownloadSpeedEvent event) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  gtk_label_set_label(GTK_LABEL(data->download_speed), event.speed);
}

void network_refresh_network_list(GtkWidget *self, IO_NetworkListEvent event) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  network_popover_refresh(data->popover, event.list);
}

void network_emit_settings_clicked(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->on_settings_clicked();
}
void network_emit_ping_clicked(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->on_ping_clicked();
}
void network_emit_network_clicked(GtkWidget *self, const char *address) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->on_address_clicked(address);
}
