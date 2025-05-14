#include "ui/include/ping.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

struct _Ping {
  GtkWindow parent_instance;
};

G_DEFINE_TYPE(Ping, ping, BASE_WINDOW_TYPE)

static void ping_class_init(PingClass *) {}

static void ping_init(Ping *self) {
  char *command[] = {"ping", "8.8.8.8", NULL};
  vte_window(GTK_WINDOW(self), command);
}

GtkWidget *ping_new(GtkApplication *app) {
  return g_object_new(PING_TYPE,
                      //
                      "application", app,
                      //
                      "name", "PingWindow",
                      //
                      "width-request", 1000,
                      //
                      "height-request", 700,
                      //
                      "toggle-on-escape", true,
                      //
                      "layer", GTK_LAYER_SHELL_LAYER_OVERLAY,
                      //
                      "layer-namespace", "LayerShell/Ping",
                      //
                      "layer-keyboard-mode",
                      GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE,
                      //
                      NULL);
}
