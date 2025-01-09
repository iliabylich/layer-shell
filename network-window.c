#include "network-window.h"
#include "bindings.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define ns(name) network_ns_##name

GtkWindow *ns(window);

typedef struct {
  GtkWidget *wrapper;
  GtkWidget *label;
} ns(row_t);

ns(row_t) ns(rows)[5];
ns(row_t) ns(settings_row);
ns(row_t) ns(exit_row);

const uint32_t ns(WIDTH) = 700;

static ns(row_t) ns(row_new)(const char *text, const char *icon_name) {
  GtkWidget *row = gtk_center_box_new();
  gtk_widget_add_css_class(row, "widget-network-row");
  gtk_orientable_set_orientation(GTK_ORIENTABLE(row),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_widget_set_halign(row, GTK_ALIGN_FILL);

  GtkWidget *label = gtk_label_new(text);
  gtk_label_set_justify(GTK_LABEL(label), GTK_JUSTIFY_LEFT);
  gtk_label_set_xalign(GTK_LABEL(label), 0.0);
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(row), label);

  GtkWidget *image = gtk_image_new();
  gtk_image_set_from_icon_name(GTK_IMAGE(image), icon_name);
  gtk_image_set_icon_size(GTK_IMAGE(image), GTK_ICON_SIZE_LARGE);
  gtk_image_set_pixel_size(GTK_IMAGE(image), 30);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(row), image);

  return (ns(row_t)){.wrapper = row, .label = label};
}

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(ns(window)), "NetworksWindow");
  window_set_width_request(ns(window), ns(WIDTH));

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(layout, "widget-network-row-list");
  gtk_window_set_child(ns(window), layout);

  for (size_t i = 0; i < 5; i++) {
    ns(row_t) row = ns(row_new)("--", "edit-copy");
    gtk_box_append(GTK_BOX(layout), row.wrapper);
    ns(rows)[i] = row;
  }

  ns(settings_row) =
      ns(row_new)("Settings (nmtui)", "preferences-system-network");
  gtk_box_append(GTK_BOX(layout), ns(settings_row).wrapper);

  ns(exit_row) = ns(row_new)("Close", "window-close");
  gtk_box_append(GTK_BOX(layout), ns(exit_row).wrapper);
}

static void ns(toggle)(void) { flip_window_visibility(ns(window)); }

static void ns(move)(uint32_t margin_left, uint32_t margin_top) {
  move_layer_window(ns(window), margin_left, margin_top);
}

static void ns(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                             GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    ns(toggle)();
  }
}

static void ns(row_set_on_click)(ns(row_t) row, GCallback callback,
                                 void *data) {
  GtkGestureClick *ctrl = GTK_GESTURE_CLICK(gtk_gesture_click_new());
  g_signal_connect(ctrl, "pressed", callback, data);
  gtk_widget_add_controller(row.wrapper, GTK_EVENT_CONTROLLER(ctrl));
}

static void ns(settings_row_on_click)(void) {
  ns(toggle)();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnNetworkEditor});
}

typedef struct {
  size_t row_idx;
  char *text;
} ns(row_checkpoint_t);

static ns(row_checkpoint_t) *
    ns(row_checkpoint_new)(size_t row_idx, const char *text) {
  size_t len = strlen(text);
  char *copy = malloc(len + 1);
  memcpy(copy, text, len);
  copy[len] = 0;

  ns(row_checkpoint_t) *safepoint = malloc(sizeof(ns(row_checkpoint_t)));
  safepoint->row_idx = row_idx;
  safepoint->text = copy;
  return safepoint;
}

static void ns(row_checkpoint_free)(ns(row_checkpoint_t) * safepoint) {
  free(safepoint->text);
  free(safepoint);
}

static void ns(row_restore_label)(gpointer user_data) {
  ns(row_checkpoint_t) *safepoint = (ns(row_checkpoint_t) *)user_data;
  GtkWidget *label = ns(rows)[safepoint->row_idx].label;
  gtk_label_set_label(GTK_LABEL(label), safepoint->text);
  ns(row_checkpoint_free)(safepoint);
}

static void ns(row_on_click)(GtkGestureClick *, gint, gdouble, gdouble,
                             gpointer user_data) {
  size_t row_idx = (size_t)(user_data);
  ns(row_t) row = ns(rows)[row_idx];
  const char *ip = gtk_widget_get_tooltip_text(row.label);
  const char *label = gtk_label_get_label(GTK_LABEL(row.label));
  ns(row_checkpoint_t) *safepoint = ns(row_checkpoint_new)(row_idx, label);

  GdkDisplay *display = gdk_display_get_default();
  GdkClipboard *clipboard = gdk_display_get_clipboard(display);
  gdk_clipboard_set_text(clipboard, ip);

  gtk_label_set_label(GTK_LABEL(row.label), "Copied!");
  g_timeout_add_seconds_once(1, ns(row_restore_label), safepoint);
}

static void ns(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case NetworkList: {
    LAYER_SHELL_IO_CArray_Network networks = event->network_list.list;
    for (size_t i = 0; i < 5; i++) {
      ns(row_t) row = ns(rows)[i];
      if (i < networks.len) {
        LAYER_SHELL_IO_Network network = networks.ptr[i];
        gtk_widget_set_visible(row.wrapper, true);
        char buffer[100];
        sprintf(buffer, "%s: %s", network.iface.ptr, network.address.ptr);
        gtk_label_set_label(GTK_LABEL(row.label), buffer);
        gtk_widget_set_tooltip_text(row.label, network.address.ptr);
      } else {
        gtk_widget_set_visible(row.wrapper, false);
      }
    }
    break;
  }
  default:
    break;
  }
}

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(ns(window), "LayerShell/Networks");
  gtk_layer_set_keyboard_mode(ns(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  ns(row_set_on_click)(ns(settings_row), G_CALLBACK(ns(settings_row_on_click)),
                       NULL);
  ns(row_set_on_click)(ns(exit_row), G_CALLBACK(ns(toggle)), NULL);

  for (size_t i = 0; i < 5; i++) {
    ns(row_t) row = ns(rows)[i];
    ns(row_set_on_click)(row, G_CALLBACK(ns(row_on_click)), (void *)i);
  }

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(ns(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(ns(window)), ctrl);

  layer_shell_io_subscribe(ns(on_io_event));
}

static uint32_t ns(width)(void) { return ns(WIDTH); }

window_t NETWORK = {.init = ns(init),
                    .toggle = ns(toggle),
                    .activate = ns(activate),
                    .move = ns(move),
                    .width = ns(width)};
