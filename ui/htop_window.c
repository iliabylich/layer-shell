#include "ui/htop_window.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

LOGGER("HTopWindow", 0)

struct _HTopWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(HTopWindow, htop_window, BASE_WINDOW_TYPE)

static void htop_window_init(HTopWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_object_set(G_OBJECT(self), "width-request", 1000, "height-request", 700,
               NULL);

  base_window_set_toggle_on_escape(BASE_WINDOW(self));
  base_window_vte(BASE_WINDOW(self), (char *[]){"htop", NULL});
}

static void htop_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(htop_window_parent_class)->dispose(object);
}

static void htop_window_class_init(HTopWindowClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = htop_window_dispose;
}

GtkWidget *htop_window_new(GtkApplication *app) {
  return g_object_new(htop_window_get_type(), "application", app, NULL);
}

void htop_window_toggle(HTopWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}
