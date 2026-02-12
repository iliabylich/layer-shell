#include "ui/terminal_window.h"
#include "bindings.h"
#include "ui/logger.h"

extern const IO_IOConfig *config;

LOGGER("TerminalWindow", 0)

struct _TerminalWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(TerminalWindow, terminal_window, BASE_WINDOW_TYPE)

static void terminal_window_init(TerminalWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  base_window_vte(BASE_WINDOW(self), config->terminal.command);
}

static void terminal_window_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), terminal_window_get_type());
  G_OBJECT_CLASS(terminal_window_parent_class)->dispose(object);
}

static void terminal_window_class_init(TerminalWindowClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = terminal_window_dispose;

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/terminal_window.ui");
}

GtkWidget *terminal_window_new(GtkApplication *app) {
  return g_object_new(terminal_window_get_type(), "application", app, NULL);
}

void terminal_window_toggle(TerminalWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}
