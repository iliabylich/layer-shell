#include "ui/view_models/caps_lock_model.h"

struct _CapsLockModel {
  GObject parent_instance;
  gboolean enabled;
};

G_DEFINE_TYPE(CapsLockModel, caps_lock_model, G_TYPE_OBJECT)

enum {
  PROP_ENABLED = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void caps_lock_model_get_property(GObject *object, guint property_id,
                                         GValue *value, GParamSpec *pspec) {
  CapsLockModel *self = CAPS_LOCK_MODEL(object);
  switch (property_id) {
  case PROP_ENABLED:
    g_value_set_boolean(value, self->enabled);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void caps_lock_model_set_property(GObject *object, guint property_id,
                                         const GValue *value,
                                         GParamSpec *pspec) {
  CapsLockModel *self = CAPS_LOCK_MODEL(object);
  switch (property_id) {
  case PROP_ENABLED:
    self->enabled = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_ENABLED]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void caps_lock_model_init(CapsLockModel *self) { self->enabled = false; }

static void caps_lock_model_class_init(CapsLockModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = caps_lock_model_get_property;
  object_class->set_property = caps_lock_model_set_property;

  properties[PROP_ENABLED] = g_param_spec_boolean(
      "enabled", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

CapsLockModel *caps_lock_model_new(void) {
  return g_object_new(caps_lock_model_get_type(), NULL);
}
