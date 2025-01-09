#include "network-window.h"
#include "bindings.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

GtkWindow *network_window;
typedef struct {
  GtkCenterBox *wrapper;
  GtkLabel *label;
} network_row_t;
network_row_t networks_rows[5];
network_row_t network_settings_row;
network_row_t network_exit_row;

const uint32_t NETWORK_WINDOW_WIDTH = 700;

static network_row_t network_row_new(const char *text, const char *icon_name) {
  GtkCenterBox *row = GTK_CENTER_BOX(gtk_center_box_new());
  gtk_widget_add_css_class(GTK_WIDGET(row), "widget-network-row");
  gtk_orientable_set_orientation(GTK_ORIENTABLE(row),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_widget_set_halign(GTK_WIDGET(row), GTK_ALIGN_FILL);

  GtkLabel *label = GTK_LABEL(gtk_label_new(text));
  gtk_label_set_justify(label, GTK_JUSTIFY_LEFT);
  gtk_label_set_xalign(label, 0.0);
  gtk_center_box_set_start_widget(row, GTK_WIDGET(label));

  GtkImage *image = GTK_IMAGE(gtk_image_new());
  gtk_image_set_from_icon_name(image, icon_name);
  gtk_image_set_icon_size(image, GTK_ICON_SIZE_LARGE);
  gtk_image_set_pixel_size(image, 30);
  gtk_center_box_set_end_widget(row, GTK_WIDGET(image));

  return (network_row_t){.wrapper = row, .label = label};
}

static void network_window_init(void) {
  network_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(network_window), "NetworksWindow");
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, NETWORK_WINDOW_WIDTH);
  g_object_set_property(G_OBJECT(network_window), "width-request",
                        &width_request);

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(layout), "widget-network-row-list");
  gtk_window_set_child(network_window, GTK_WIDGET(layout));

  for (size_t i = 0; i < 5; i++) {
    network_row_t row = network_row_new("--", "edit-copy");
    gtk_box_append(layout, GTK_WIDGET(row.wrapper));
    networks_rows[i] = row;
  }

  network_settings_row =
      network_row_new("Settings (nmtui)", "preferences-system-network");
  gtk_box_append(layout, GTK_WIDGET(network_settings_row.wrapper));

  network_exit_row = network_row_new("Close", "window-close");
  gtk_box_append(layout, GTK_WIDGET(network_exit_row.wrapper));
}

static void network_window_toggle(void) {
  flip_window_visibility(network_window);
}

static void network_window_move(uint32_t margin_left, uint32_t margin_top) {
  move_layer_window(network_window, margin_left, margin_top);
}

static void network_window_on_key_press(GtkEventControllerKey *, guint keyval,
                                        guint, GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    network_window_toggle();
  }
}

static void network_row_set_on_click(network_row_t row, GCallback callback,
                                     void *data) {
  GtkGestureClick *ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  g_signal_connect(ctrl, "pressed", callback, data);
  gtk_widget_add_controller(GTK_WIDGET(row.wrapper),
                            GTK_EVENT_CONTROLLER(ctrl));
}

static void on_network_settings_row_click(void) {
  network_window_toggle();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnNetworkEditor});
}

typedef struct {
  size_t row_idx;
  char *text;
} network_row_safe_point_t;

static network_row_safe_point_t *network_row_safe_point_new(size_t row_idx,
                                                            const char *text) {
  size_t len = strlen(text);
  char *copy = malloc(len + 1);
  memcpy(copy, text, len);
  copy[len] = 0;

  network_row_safe_point_t *safepoint =
      malloc(sizeof(network_row_safe_point_t));
  safepoint->row_idx = row_idx;
  safepoint->text = copy;
  return safepoint;
}

static void network_row_safe_point_free(network_row_safe_point_t *safepoint) {
  free(safepoint->text);
  free(safepoint);
}

static void network_row_restore_label(gpointer user_data) {
  network_row_safe_point_t *safepoint = (network_row_safe_point_t *)user_data;
  GtkLabel *label = networks_rows[safepoint->row_idx].label;
  gtk_label_set_label(label, safepoint->text);
  network_row_safe_point_free(safepoint);
}

static void network_row_on_click(GtkGestureClick *, gint, gdouble, gdouble,
                                 gpointer user_data) {
  size_t row_idx = (size_t)(user_data);
  network_row_t row = networks_rows[row_idx];
  const char *ip = gtk_widget_get_tooltip_text(GTK_WIDGET(row.label));
  const char *label = gtk_label_get_label(row.label);
  network_row_safe_point_t *safepoint =
      network_row_safe_point_new(row_idx, label);

  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  gtk_label_set_label(row.label, "Copied!");
  g_timeout_add_seconds_once(1, network_row_restore_label, safepoint);
}

static void network_window_on_io_event(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case NetworkList: {
    LAYER_SHELL_IO_CArray_Network networks = event->network_list.list;
    for (size_t i = 0; i < 5; i++) {
      network_row_t row = networks_rows[i];
      if (i < networks.len) {
        LAYER_SHELL_IO_Network network = networks.ptr[i];
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), true);
        char buffer[100];
        sprintf(buffer, "%s: %s", network.iface.ptr, network.address.ptr);
        gtk_label_set_label(row.label, buffer);
        gtk_widget_set_tooltip_text(GTK_WIDGET(row.label), network.address.ptr);
      } else {
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), false);
      }
    }
    break;
  }
  default:
    break;
  }
}

static void network_window_activate(GApplication *app) {
  gtk_window_set_application(network_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(network_window);
  gtk_layer_set_layer(network_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(network_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(network_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(network_window, "LayerShell/Networks");
  gtk_layer_set_keyboard_mode(network_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  network_row_set_on_click(network_settings_row,
                           G_CALLBACK(on_network_settings_row_click), NULL);
  network_row_set_on_click(network_exit_row, G_CALLBACK(network_window_toggle),
                           NULL);

  for (size_t i = 0; i < 5; i++) {
    network_row_t row = networks_rows[i];
    network_row_set_on_click(row, G_CALLBACK(network_row_on_click), (void *)i);
  }

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(network_window_on_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(network_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  layer_shell_io_subscribe(network_window_on_io_event);
}

static uint32_t network_window_width(void) { return NETWORK_WINDOW_WIDTH; }

window_t NETWORK = {.init = network_window_init,
                    .toggle = network_window_toggle,
                    .activate = network_window_activate,
                    .move = network_window_move,
                    .width = network_window_width};
