#include "ui/terminal_window.h"
#include "bindings.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

extern const IO_IOConfig *config;

LOGGER("TerminalWindow", 0)

struct _TerminalWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(TerminalWindow, terminal_window, BASE_WINDOW_TYPE)

static void terminal_window_init(TerminalWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Terminal");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_object_set(G_OBJECT(self), "width-request", 1000, "height-request", 700,
               NULL);

  base_window_set_toggle_on_escape(BASE_WINDOW(self));
  base_window_vte(BASE_WINDOW(self), config->terminal.command);
}

static void terminal_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(terminal_window_parent_class)->dispose(object);
}

static void terminal_window_class_init(TerminalWindowClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = terminal_window_dispose;
}

GtkWidget *terminal_window_new(GtkApplication *app) {
  return g_object_new(terminal_window_get_type(), "application", app, NULL);
}

void terminal_window_toggle(TerminalWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}
