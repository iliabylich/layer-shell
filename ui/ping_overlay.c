#include "ui/ping_overlay.h"
#include "bindings.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"

extern const IO_IOConfig *config;

LOGGER("PingOverlay", 0)

struct _PingOverlay {
  GtkWidget parent_instance;

  IOModel *model;
};

G_DEFINE_TYPE(PingOverlay, ping_overlay, BASE_OVERLAY_TYPE)

enum {
  PROP_MODEL = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void toggle_requested(BaseOverlay *, gpointer data) {
  PingOverlay *self = PING_OVERLAY(data);
  gobject_toggle_nested(G_OBJECT(self->model), "overlays", "ping");
}

static void ping_overlay_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  PingOverlay *self = PING_OVERLAY(object);
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void ping_overlay_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  PingOverlay *self = PING_OVERLAY(object);
  switch (property_id) {
  case PROP_MODEL:
    g_set_object(&self->model, g_value_get_object(value));
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void ping_overlay_init(PingOverlay *self) {
  LOG("init");
  self->model = NULL;
  gtk_widget_init_template(GTK_WIDGET(self));
  base_overlay_vte(BASE_OVERLAY(self), config->ping);
  g_signal_connect(self, "toggle-requested", G_CALLBACK(toggle_requested),
                   self);
}

static void ping_overlay_dispose(GObject *object) {
  LOG("dispose");
  PingOverlay *self = PING_OVERLAY(object);
  g_clear_object(&self->model);
  gtk_widget_dispose_template(GTK_WIDGET(object), ping_overlay_get_type());
  G_OBJECT_CLASS(ping_overlay_parent_class)->dispose(object);
}

static void ping_overlay_class_init(PingOverlayClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = ping_overlay_get_property;
  object_class->set_property = ping_overlay_set_property;
  object_class->dispose = ping_overlay_dispose;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/ping_overlay.ui");
}

GtkWidget *ping_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(ping_overlay_get_type(), "application", app, "model",
                      model, NULL);
}
