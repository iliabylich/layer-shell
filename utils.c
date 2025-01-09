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

void window_set_width_request(GtkWindow *window, uint32_t width) {
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, width);
  g_object_set_property(G_OBJECT(window), "width-request", &width_request);
}

void window_set_height_request(GtkWindow *window, uint32_t height) {
  GValue height_request = G_VALUE_INIT;
  g_value_init(&height_request, G_TYPE_INT);
  g_value_set_int(&height_request, height);
  g_object_set_property(G_OBJECT(window), "height-request", &height_request);
}
