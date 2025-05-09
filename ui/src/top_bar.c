#include "ui/include/top_bar.h"
#include <gtk4-layer-shell.h>

struct _TopBar {
  GtkWindow parent_instance;

  GtkBox *left;
  GtkBox *right;
};

G_DEFINE_TYPE(TopBar, top_bar, GTK_TYPE_WINDOW)

static void top_bar_class_init(TopBarClass *) {}

static void top_bar_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(window, "LayerShell/TopBar");
  gtk_layer_auto_exclusive_zone_enable(window);
}

static void top_bar_init(TopBar *self) {
  gtk_widget_set_name(GTK_WIDGET(self), "TopBarWindow");
  gtk_widget_add_css_class(GTK_WIDGET(self), "top-bar-window");
  top_bar_init_layer(GTK_WINDOW(self));

  GtkWidget *layout = gtk_center_box_new();
  gtk_widget_add_css_class(layout, "wrapper");
  gtk_window_set_child(GTK_WINDOW(self), layout);

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);

  gtk_center_box_set_start_widget(GTK_CENTER_BOX(layout), left);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(layout), right);

  self->left = GTK_BOX(left);
  self->right = GTK_BOX(right);
}

TopBar *top_bar_new(GtkApplication *app) {
  return g_object_new(top_bar_get_type(), "application", app, NULL);
}

void top_bar_push_left(TopBar *top_bar, GtkWidget *child) {
  gtk_box_append(GTK_BOX(top_bar->left), child);
}

void top_bar_push_right(TopBar *top_bar, GtkWidget *child) {
  gtk_box_append(GTK_BOX(top_bar->right), child);
}
