#include "ui/include/htop.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

struct _Htop {
  GtkWindow parent_instance;
};

G_DEFINE_TYPE(Htop, htop, GTK_TYPE_WINDOW)

static void htop_class_init(HtopClass *) {}

static void htop_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(window, "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
}

static void htop_init(Htop *self) {
  gtk_widget_set_name(GTK_WIDGET(self), "HtopWindow");
  gtk_widget_set_size_request(GTK_WIDGET(self), 1000, 700);
  window_toggle_on_escape(GTK_WINDOW(self));
  htop_init_layer(GTK_WINDOW(self));
  char *command[] = {"htop", NULL};
  vte_window(GTK_WINDOW(self), command);
}

Htop *htop_new(GtkApplication *app) {
  return g_object_new(htop_get_type(), "application", app, NULL);
}
