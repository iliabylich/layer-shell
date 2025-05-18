#include "ui/include/htop.h"
#include "ui/include/builder.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

GtkWidget *htop_init(GtkApplication *app) {
  GtkWidget *self = htop_get_widget("HTOP");
  gtk_window_set_application(GTK_WINDOW(self), app);
  window_set_toggle_on_escape(GTK_WINDOW(self));
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
  window_vte(GTK_WINDOW(self), (char *[]){"htop", NULL});

  return self;
}

void htop_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }
