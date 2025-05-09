#include "ui/include/top_bar/network.h"
#include "ui/include/icons.h"
#include "ui/include/top_bar/network_popover.h"

struct _Network {
  GtkButton parent_instance;

  GtkWidget *label;
  GtkWidget *image;
  GtkWidget *download_speed_label;
  GtkWidget *download_speed_icon;
  GtkWidget *upload_speed_label;
  GtkWidget *upload_speed_icon;
  GtkWidget *popover;
};

G_DEFINE_TYPE(Network, network, GTK_TYPE_BUTTON)

enum {
  SETTINGS_CLICKED = 0,
  PING_CLICKED,
  NETWORK_CLICKED,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void network_class_init(NetworkClass *klass) {
  signals[SETTINGS_CLICKED] =
      g_signal_new("settings-clicked", G_TYPE_FROM_CLASS(klass),
                   G_SIGNAL_RUN_LAST, 0, NULL, NULL, NULL, G_TYPE_NONE, 0);

  signals[PING_CLICKED] =
      g_signal_new("ping-clicked", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST,
                   0, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[NETWORK_CLICKED] = g_signal_new(
      "network-clicked", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0, NULL,
      NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
}

static void on_click(Network *self) {
  gtk_popover_popup(GTK_POPOVER(self->popover));
}

static void network_init(Network *self) {
  gtk_button_set_label(GTK_BUTTON(self), "--");
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "network");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(self), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "Network");

  self->label = gtk_label_new("-- ");
  self->image = gtk_image_new_from_gicon(get_wifi_icon());

  self->download_speed_label = gtk_label_new("??");
  gtk_widget_add_css_class(GTK_WIDGET(self->download_speed_label),
                           "network-speed-label");
  self->download_speed_icon =
      gtk_image_new_from_gicon(get_download_speed_icon());

  self->upload_speed_label = gtk_label_new("??");
  gtk_widget_add_css_class(GTK_WIDGET(self->upload_speed_label),
                           "network-speed-label");
  self->upload_speed_icon = gtk_image_new_from_gicon(get_upload_speed_icon());

  GtkWidget *wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_button_set_child(GTK_BUTTON(self), wrapper);
  gtk_box_append(GTK_BOX(wrapper), self->label);
  gtk_box_append(GTK_BOX(wrapper), self->image);

  GtkWidget *sep = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
  gtk_box_append(GTK_BOX(wrapper), sep);

  gtk_box_append(GTK_BOX(wrapper), self->download_speed_label);
  gtk_box_append(GTK_BOX(wrapper), self->download_speed_icon);
  gtk_box_append(GTK_BOX(wrapper), self->upload_speed_label);
  gtk_box_append(GTK_BOX(wrapper), self->upload_speed_icon);

  self->popover = network_popover_new(self);
  gtk_widget_set_parent(GTK_WIDGET(self->popover), GTK_WIDGET(self));

  g_signal_connect(self, "clicked", G_CALLBACK(on_click), NULL);
}

GtkWidget *network_new() { return g_object_new(network_get_type(), NULL); }

void network_refresh_wifi_status(Network *network,
                                 IO_COption_WifiStatus wifi_status) {
  if (wifi_status.tag == IO_COption_WifiStatus_None_WifiStatus) {
    gtk_widget_set_visible(GTK_WIDGET(network->image), false);
    gtk_label_set_label(GTK_LABEL(network->label), "Not connected");
  } else {
    gtk_widget_set_visible(GTK_WIDGET(network->image), true);
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", wifi_status.some.ssid,
            wifi_status.some.strength);
    gtk_label_set_label(GTK_LABEL(network->label), buffer);
  }
}

void network_refresh_network_speed(Network *network, IO_CString upload_speed,
                                   IO_CString download_speed) {
  gtk_label_set_label(GTK_LABEL(network->upload_speed_label), upload_speed);
  gtk_label_set_label(GTK_LABEL(network->download_speed_label), download_speed);
}

void network_refresh_network_list(Network *network, IO_CArray_Network list) {
  network_popover_refresh(network->popover, list);
}

void network_emit_settings_clicked(Network *network) {
  g_signal_emit(network, signals[SETTINGS_CLICKED], 0);
}
void network_emit_ping_clicked(Network *network) {
  g_signal_emit(network, signals[PING_CLICKED], 0);
}
void network_emit_network_clicked(Network *network, const char *address) {
  g_signal_emit(network, signals[NETWORK_CLICKED], 0, address);
}
