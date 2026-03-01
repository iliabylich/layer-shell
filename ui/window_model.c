#include "ui/window_model.h"

typedef struct {
  gboolean visible;
} WindowModelPrivate;

G_DEFINE_TYPE_WITH_PRIVATE(WindowModel, window_model, G_TYPE_OBJECT)

enum {
  PROP_VISIBLE = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void window_model_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  WindowModel *self = WINDOW_MODEL(object);
  WindowModelPrivate *priv = window_model_get_instance_private(self);
  switch (property_id) {
  case PROP_VISIBLE:
    g_value_set_boolean(value, priv->visible);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void window_model_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  WindowModel *self = WINDOW_MODEL(object);
  WindowModelPrivate *priv = window_model_get_instance_private(self);
  switch (property_id) {
  case PROP_VISIBLE:
    priv->visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_VISIBLE]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void window_model_init(WindowModel *self) {
  WindowModelPrivate *priv = window_model_get_instance_private(self);
  priv->visible = false;
}

static void window_model_class_init(WindowModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = window_model_get_property;
  object_class->set_property = window_model_set_property;

  properties[PROP_VISIBLE] =
      g_param_spec_boolean("visible", NULL, NULL, false,
                           G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WindowModel *window_model_new(void) {
  return g_object_new(window_model_get_type(), NULL);
}
