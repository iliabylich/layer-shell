#include "ui/include/ping.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

struct _Ping {
  GtkWindow parent_instance;
};

G_DEFINE_TYPE(Ping, ping, GTK_TYPE_WINDOW)

static void ping_class_init(PingClass *) {}

static void ping_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(window, "LayerShell/Ping");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
}

static void ping_init(Ping *self) {
  window_toggle_on_escape(GTK_WINDOW(self));
  ping_init_layer(GTK_WINDOW(self));
  char *command[] = {"ping", "8.8.8.8", NULL};
  vte_window(GTK_WINDOW(self), command);
}

Ping *ping_new(GtkApplication *app) {
  return g_object_new(ping_get_type(),
                      //
                      "application", app,
                      //
                      "name", "PingWindow",
                      //
                      "width-request", 1000,
                      //
                      "height-request", 700,
                      //
                      NULL);
}
