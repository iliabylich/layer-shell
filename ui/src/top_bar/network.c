#include "ui/include/top_bar/network.h"
#include "gtk/gtk.h"
#include "ui/include/icons.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar.h"
#include "ui/include/top_bar/network_popover.h"

typedef struct {
  GtkWidget *label;
  GtkWidget *image;
  GtkWidget *download_speed_label;
  GtkWidget *download_speed_icon;
  GtkWidget *upload_speed_label;
  GtkWidget *upload_speed_icon;
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
  GtkWidget *self = top_bar_get_widget_by_id("NETWORK");
  GtkWidget *label = top_bar_get_widget_by_id("NETWORK_NAME_LABEL");
  GtkWidget *image = top_bar_get_widget_by_id("NETWORK_IMAGE");
  GtkWidget *download_speed_label =
      top_bar_get_widget_by_id("NETWORK_DOWNLOAD_SPEED_LABEL");
  GtkWidget *download_speed_icon =
      top_bar_get_widget_by_id("NETWORK_DOWNLOAD_SPEED_ICON");
  GtkWidget *upload_speed_label =
      top_bar_get_widget_by_id("NETWORK_UPLOAD_SPEED_LABEL");
  GtkWidget *upload_speed_icon =
      top_bar_get_widget_by_id("NETWORK_UPLOAD_SPEED_ICON");
  GtkWidget *popover = network_popover_new(self);

  data_t *data = malloc(sizeof(data_t));
  data->label = label;
  data->image = image;
  data->download_speed_label = download_speed_label;
  data->download_speed_icon = download_speed_icon;
  data->upload_speed_label = upload_speed_label;
  data->upload_speed_icon = upload_speed_icon;
  data->popover = popover;
  data->on_settings_clicked = on_settings_clicked;
  data->on_ping_clicked = on_ping_clicked;
  data->on_address_clicked = on_address_clicked;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  gtk_image_set_from_gicon(GTK_IMAGE(image), get_wifi_icon());
  gtk_image_set_from_gicon(GTK_IMAGE(download_speed_icon),
                           get_download_speed_icon());
  gtk_image_set_from_gicon(GTK_IMAGE(upload_speed_icon),
                           get_upload_speed_icon());
  gtk_widget_set_parent(GTK_WIDGET(popover), GTK_WIDGET(self));
  g_signal_connect(self, "clicked", G_CALLBACK(on_click), NULL);

  return self;
}

void network_refresh_wifi_status(GtkWidget *self,
                                 IO_COption_WifiStatus wifi_status) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  if (wifi_status.tag == IO_COption_WifiStatus_None_WifiStatus) {
    gtk_widget_set_visible(GTK_WIDGET(data->image), false);
    gtk_label_set_label(GTK_LABEL(data->label), "Not connected");
  } else {
    gtk_widget_set_visible(GTK_WIDGET(data->image), true);
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", wifi_status.some.ssid,
            wifi_status.some.strength);
    gtk_label_set_label(GTK_LABEL(data->label), buffer);
  }
}

void network_refresh_network_speed(GtkWidget *self, const char *upload_speed,
                                   const char *download_speed) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  gtk_label_set_label(GTK_LABEL(data->upload_speed_label), upload_speed);
  gtk_label_set_label(GTK_LABEL(data->download_speed_label), download_speed);
}

void network_refresh_network_list(GtkWidget *self, IO_CArray_Network list) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  network_popover_refresh(data->popover, list);
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
