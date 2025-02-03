#include "network.h"
#include "bindings.h"
#include "utils/icons.h"
#include "windows/network.h"
#include "windows/top-bar.h"
#include <gtk/gtk.h>

#define _(name) network_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);
static GtkWidget *_(image);
static GtkWidget *_(download_speed_label);
static GtkWidget *_(download_speed_icon);
static GtkWidget *_(upload_speed_label);
static GtkWidget *_(upload_speed_icon);

static GtkWidget *_(init)(void) {
  _(label) = gtk_label_new("--");

  _(image) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(image)), get_wifi_icon());

  _(download_speed_label) = gtk_label_new("??");
  gtk_widget_add_css_class(_(download_speed_label), "network-speed-label");

  _(download_speed_icon) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(download_speed_icon)),
                           get_download_speed_icon());

  _(upload_speed_label) = gtk_label_new("??");
  gtk_widget_add_css_class(_(upload_speed_label), "network-speed-label");

  _(upload_speed_icon) = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(_(upload_speed_icon)),
                           get_upload_speed_icon());

  GtkWidget *network_wrapper = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_append(GTK_BOX(network_wrapper), _(label));
  gtk_box_append(GTK_BOX(network_wrapper), _(image));

  GtkWidget *sep = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
  gtk_box_append(GTK_BOX(network_wrapper), sep);

  gtk_box_append(GTK_BOX(network_wrapper), _(download_speed_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(download_speed_icon));
  gtk_box_append(GTK_BOX(network_wrapper), _(upload_speed_label));
  gtk_box_append(GTK_BOX(network_wrapper), _(upload_speed_icon));

  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "network");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_widget_set_name(_(widget), "Network");
  gtk_widget_set_cursor(_(widget), gdk_cursor_new_from_name("pointer", NULL));
  gtk_button_set_child(GTK_BUTTON(_(widget)), network_wrapper);

  return _(widget);
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_WifiStatus: {
    if (event->wifi_status.wifi_status.tag ==
        IO_COption_WifiStatus_None_WifiStatus) {
      gtk_widget_set_visible(_(image), false);
      gtk_label_set_label(GTK_LABEL(_(label)), "Not connected");
    } else {
      gtk_widget_set_visible(_(image), true);
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wifi_status.wifi_status.some.ssid,
              event->wifi_status.wifi_status.some.strength);
      gtk_label_set_label(GTK_LABEL(_(label)), buffer);
    }
    break;
  }
  case IO_Event_NetworkSpeed: {
    gtk_label_set_label(GTK_LABEL(_(download_speed_label)),
                        event->network_speed.download_speed);
    gtk_label_set_label(GTK_LABEL(_(upload_speed_label)),
                        event->network_speed.upload_speed);
    break;
  }
  default: {
    break;
  }
  }
}

static void _(on_click)(void) {
  graphene_point_t bottom_right;
  if (!bottom_right_point_of(_(widget), TOP_BAR.window(), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the network widget");
    return;
  }
  int margin_left = bottom_right.x - NETWORK.width;
  int margin_top = bottom_right.y;
  NETWORK.move(margin_left, margin_top);

  NETWORK.toggle();
}

static void _(activate)(void) {
  g_signal_connect(_(widget), "clicked", _(on_click), NULL);

  layer_shell_io_subscribe(_(on_io_event));
}

widget_t NETWORK_WIDGET = {.init = _(init), .activate = _(activate)};
