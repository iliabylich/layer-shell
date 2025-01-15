#include "utils.h"
#include <gtk4-layer-shell.h>

void flip_window_visibility(GtkWindow *window) {
  gtk_widget_set_visible(GTK_WIDGET(window),
                         !gtk_widget_get_visible(GTK_WIDGET(window)));
}

void move_layer_window(GtkWindow *window, int x, int y) {
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_LEFT, x);
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, y);
}

void window_set_width_request(GtkWindow *window, int width) {
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, width);
  g_object_set_property(G_OBJECT(window), "width-request", &width_request);
}

void window_set_height_request(GtkWindow *window, int height) {
  GValue height_request = G_VALUE_INIT;
  g_value_init(&height_request, G_TYPE_INT);
  g_value_set_int(&height_request, height);
  g_object_set_property(G_OBJECT(window), "height-request", &height_request);
}

bool bottom_right_point_of(GtkWidget *widget, GtkWindow *window,
                           graphene_point_t *out) {
  graphene_rect_t bounds;
  if (!gtk_widget_compute_bounds(widget, GTK_WIDGET(window), &bounds)) {
    return false;
  }

  out->x = bounds.origin.x + bounds.size.width;
  out->y = bounds.origin.y + bounds.size.height;

  return true;
}
