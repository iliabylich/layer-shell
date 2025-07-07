#include "ui/top_bar.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

LOGGER("TopBar", 0)

struct _TopBar {
  GtkWidget parent_instance;

  GtkWidget *left;
  GtkWidget *right;
};

G_DEFINE_TYPE(TopBar, top_bar, GTK_TYPE_WINDOW)

static void top_bar_init(TopBar *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/TopBar");
  gtk_layer_auto_exclusive_zone_enable(GTK_WINDOW(self));

  gtk_widget_add_css_class(GTK_WIDGET(self), "top-bar-window");

  self->left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
  self->right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);

  GtkWidget *child = gtk_center_box_new();
  gtk_widget_add_css_class(GTK_WIDGET(child), "wrapper");
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(child), self->left);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(child), self->right);
  gtk_widget_set_halign(child, GTK_ALIGN_FILL);
  gtk_window_set_child(GTK_WINDOW(self), child);
}

static void top_bar_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(top_bar_parent_class)->dispose(object);
}

static void top_bar_class_init(TopBarClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = top_bar_dispose;
}

GtkWidget *top_bar_new(GtkApplication *app) {
  return g_object_new(top_bar_get_type(), "application", app, NULL);
}

void top_bar_push_left(TopBar *self, GtkWidget *child) {
  gtk_box_append(GTK_BOX(self->left), child);
}

void top_bar_push_right(TopBar *self, GtkWidget *child) {
  gtk_box_append(GTK_BOX(self->right), child);
}
