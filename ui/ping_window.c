#include "ui/ping_window.h"
#include "bindings.h"
#include "ui/logger.h"

extern const IO_IOConfig *config;

LOGGER("PingWindow", 0)

struct _PingWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(PingWindow, ping_window, BASE_WINDOW_TYPE)

static void ping_window_init(PingWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  base_window_vte(BASE_WINDOW(self), config->ping);
}

static void ping_window_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), ping_window_get_type());
  G_OBJECT_CLASS(ping_window_parent_class)->dispose(object);
}

static void ping_window_class_init(PingWindowClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = ping_window_dispose;

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/ping_window.ui");
}

GtkWidget *ping_window_new(GtkApplication *app) {
  return g_object_new(ping_window_get_type(), "application", app, NULL);
}

void ping_window_toggle(PingWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}
