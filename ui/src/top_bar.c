#include "ui/include/top_bar.h"
#include <gtk4-layer-shell.h>

struct _TopBar {
  BaseWindow parent_instance;

  GtkWidget *left;
  GtkWidget *right;
};

G_DEFINE_TYPE(TopBar, top_bar, BASE_WINDOW_TYPE)

static void top_bar_class_init(TopBarClass *) {}

static void top_bar_init(TopBar *self) {
  self->left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  self->right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);

  GtkWidget *layout =
      g_object_new(GTK_TYPE_CENTER_BOX,
                   //
                   "css-classes", (const char *[]){"wrapper", NULL},
                   //
                   "start-widget", self->left,
                   //
                   "end-widget", self->right,
                   //
                   NULL);
  gtk_window_set_child(GTK_WINDOW(self), layout);
}

GtkWidget *top_bar_new(GtkApplication *app) {
  return g_object_new(TOP_BAR_TYPE,
                      //
                      "application", app,
                      //
                      "name", "TopBarWindow",
                      //
                      "css-classes", (const char *[]){"top-bar-window", NULL},
                      //
                      "layer", GTK_LAYER_SHELL_LAYER_TOP,
                      //
                      "layer-anchor-top", true,
                      //
                      "layer-anchor-left", true,
                      //
                      "layer-anchor-right", true,
                      //
                      "layer-margin-top", 0,
                      //
                      "layer-namespace", "LayerShell/TopBar",
                      //
                      "layer-auto-exclusive-zone-enabled", true,
                      //
                      NULL);
}

void top_bar_push_left(TopBar *top_bar, GtkWidget *child) {
  gtk_box_append(GTK_BOX(top_bar->left), child);
}

void top_bar_push_right(TopBar *top_bar, GtkWidget *child) {
  gtk_box_append(GTK_BOX(top_bar->right), child);
}
