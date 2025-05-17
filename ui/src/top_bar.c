#include "ui/include/top_bar.h"
#include "top_bar.xml.xxd"
#include "ui/include/macros.h"
#include <gtk4-layer-shell.h>

BLP_BUILDER(top_bar)

GtkWidget *top_bar_init(GtkApplication *app) {
  GtkWidget *self = builder_get_object("TOP_BAR");
  gtk_window_set_application(GTK_WINDOW(self), app);

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/TopBar");
  gtk_layer_auto_exclusive_zone_enable(GTK_WINDOW(self));

  return self;
}

GtkWidget *top_bar_get_widget_by_id(const char *id) {
  return builder_get_object(id);
}
