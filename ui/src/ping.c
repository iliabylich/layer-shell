#include "ui/include/ping.h"
#include "ping.xml.xxd"
#include "ui/include/macros.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

BLP_BUILDER(ping)

GtkWidget *ping_init(GtkApplication *app) {
  GtkWidget *self = builder_get_object("PING");
  gtk_window_set_application(GTK_WINDOW(self), app);
  window_set_toggle_on_escape(GTK_WINDOW(self));
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Ping");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
  window_vte(GTK_WINDOW(self), (char *[]){"ping", "8.8.8.8", NULL});

  return self;
}

void ping_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }
