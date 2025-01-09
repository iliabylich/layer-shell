#include "utils.h"
#include <gtk4-layer-shell.h>

void flip_window_visibility(GtkWindow *window) {
  gtk_widget_set_visible(GTK_WIDGET(window),
                         !gtk_widget_get_visible(GTK_WIDGET(window)));
}

void move_layer_window(GtkWindow *window, uint32_t margin_left,
                       uint32_t margin_top) {
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_LEFT, margin_left);
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, margin_top);
}
