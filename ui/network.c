#include "ui/network.h"
#include "ui/logger.h"
#include "ui/network_popover.h"

LOGGER("Network", 1)

enum {
  SIGNAL_CLICKED_SETTINGS = 0,
  SIGNAL_CLICKED_PING,
  SIGNAL_CLICKED_ADDRESS,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _Network {
  GtkWidget parent_instance;

  GtkWidget *root;
  GtkWidget *network_name_label;
  GtkWidget *download_speed_label;
  GtkWidget *upload_speed_label;
  GtkWidget *popover;
};

G_DEFINE_TYPE(Network, network, GTK_TYPE_WIDGET)

static void on_click(GtkWidget *, Network *self) {
  network_popover_open(NETWORK_POPOVER(self->popover));
}

static void on_settings_clicked(NetworkPopover *, Network *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_SETTINGS], 0);
}
static void on_ping_clicked(NetworkPopover *, Network *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_PING], 0);
}
static void on_address_clicked(NetworkPopover *, const char *address,
                               Network *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_ADDRESS], 0, address);
}

static void network_init(Network *self) {
  LOG("init");
  self->root = gtk_button_new();
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "network");
  gtk_widget_add_css_class(self->root, "padded");
  gtk_widget_add_css_class(self->root, "clickable");
  gtk_widget_set_cursor_from_name(self->root, "pointer");
  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
  g_signal_connect(self->root, "clicked", G_CALLBACK(on_click), self);

  self->network_name_label = gtk_label_new("-- ");

  GtkWidget *wifi_icon = gtk_label_new("");
  gtk_widget_add_css_class(wifi_icon, "network-icon");

  GtkWidget *separator = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
  gtk_widget_add_css_class(separator, "separator");

  self->download_speed_label = gtk_label_new("??");
  gtk_widget_add_css_class(self->download_speed_label, "network-speed-label");

  GtkWidget *download_speed_icon = gtk_label_new("󰇚");
  gtk_widget_add_css_class(download_speed_icon, "network-icon");

  self->upload_speed_label = gtk_label_new("??");
  gtk_widget_add_css_class(self->upload_speed_label, "network-speed-label");

  GtkWidget *upload_speed_icon = gtk_label_new("󰇚");
  gtk_widget_add_css_class(upload_speed_icon, "network-icon");

  GtkWidget *wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_button_set_child(GTK_BUTTON(self->root), wrapper);
  gtk_box_append(GTK_BOX(wrapper), self->network_name_label);
  gtk_box_append(GTK_BOX(wrapper), wifi_icon);
  gtk_box_append(GTK_BOX(wrapper), separator);
  gtk_box_append(GTK_BOX(wrapper), self->download_speed_label);
  gtk_box_append(GTK_BOX(wrapper), download_speed_icon);
  gtk_box_append(GTK_BOX(wrapper), self->upload_speed_label);
  gtk_box_append(GTK_BOX(wrapper), upload_speed_icon);

  self->popover = network_popover_new();
  gtk_widget_set_parent(self->popover, GTK_WIDGET(self->root));

  g_signal_connect(self->popover, "clicked-settings",
                   G_CALLBACK(on_settings_clicked), self);
  g_signal_connect(self->popover, "clicked-ping", G_CALLBACK(on_ping_clicked),
                   self);
  g_signal_connect(self->popover, "clicked-address",
                   G_CALLBACK(on_address_clicked), self);
}

static void network_dispose(GObject *object) {
  LOG("dispose");
  Network *self = NETWORK(object);

  g_clear_pointer(&self->popover, gtk_widget_unparent);
  g_clear_pointer(&self->root, gtk_widget_unparent);

  G_OBJECT_CLASS(network_parent_class)->dispose(object);
}

static void network_class_init(NetworkClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = network_dispose;

  signals[SIGNAL_CLICKED_SETTINGS] = g_signal_new_class_handler(
      "clicked-settings", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_PING] = g_signal_new_class_handler(
      "clicked-ping", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_ADDRESS] = g_signal_new_class_handler(
      "clicked-address", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *network_new(void) { return g_object_new(network_get_type(), NULL); }

void network_refresh_wifi_status(Network *self, IO_WifiStatusEvent event) {
  if (event.wifi_status.tag == IO_COption_WifiStatus_None_WifiStatus) {
    gtk_label_set_label(GTK_LABEL(self->network_name_label), "Not connected");
  } else {
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", event.wifi_status.some.ssid,
            event.wifi_status.some.strength);
    gtk_label_set_label(GTK_LABEL(self->network_name_label), buffer);
  }
}

void network_refresh_upload_speed(Network *self, IO_UploadSpeedEvent event) {
  gtk_label_set_label(GTK_LABEL(self->upload_speed_label), event.speed);
}
void network_refresh_download_speed(Network *self,
                                    IO_DownloadSpeedEvent event) {
  gtk_label_set_label(GTK_LABEL(self->download_speed_label), event.speed);
}

void network_refresh_network_list(Network *self, IO_NetworkListEvent event) {
  network_popover_refresh(NETWORK_POPOVER(self->popover), event);
}
