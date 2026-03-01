#include "ui/ping_overlay.h"
#include "bindings.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"

extern const IO_IOConfig *config;

LOGGER("PingOverlay", 0)

struct _PingOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(PingOverlay, ping_overlay, BASE_OVERLAY_TYPE)

static void toggle_requested(BaseOverlay *, gpointer data) {
  PingOverlay *self = PING_OVERLAY(data);
  gobject_toggle_nested(G_OBJECT(base_overlay_get_model(BASE_OVERLAY(self))),
                        "overlays", "ping");
}

static void ping_overlay_init(PingOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  base_overlay_vte(BASE_OVERLAY(self), config->ping);
  g_signal_connect(self, "toggle-requested", G_CALLBACK(toggle_requested),
                   self);
}

static void ping_overlay_dispose(GObject *object) {
  LOG("dispose");
  gtk_widget_dispose_template(GTK_WIDGET(object), ping_overlay_get_type());
  G_OBJECT_CLASS(ping_overlay_parent_class)->dispose(object);
}

static void ping_overlay_class_init(PingOverlayClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = ping_overlay_dispose;

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/ping_overlay.ui");
}

GtkWidget *ping_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(ping_overlay_get_type(), "application", app, "model",
                      model, NULL);
}
