#include "ui/view_models/overlays_model.h"

struct _OverlaysModel {
  GObject parent_instance;

  gboolean weather;
  gboolean session;
  gboolean terminal;
  gboolean ping;
  gboolean sound;
  gboolean caps_lock;
};

G_DEFINE_TYPE(OverlaysModel, overlays_model, G_TYPE_OBJECT)

enum {
  PROP_WEATHER = 1,
  PROP_SESSION,
  PROP_TERMINAL,
  PROP_PING,
  PROP_SOUND,
  PROP_CAPS_LOCK,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void overlays_model_get_property(GObject *object, guint property_id,
                                        GValue *value, GParamSpec *pspec) {
  OverlaysModel *self = OVERLAYS_MODEL(object);
  switch (property_id) {
  case PROP_WEATHER:
    g_value_set_boolean(value, self->weather);
    break;
  case PROP_SESSION:
    g_value_set_boolean(value, self->session);
    break;
  case PROP_TERMINAL:
    g_value_set_boolean(value, self->terminal);
    break;
  case PROP_PING:
    g_value_set_boolean(value, self->ping);
    break;
  case PROP_SOUND:
    g_value_set_boolean(value, self->sound);
    break;
  case PROP_CAPS_LOCK:
    g_value_set_boolean(value, self->caps_lock);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void overlays_model_set_property(GObject *object, guint property_id,
                                        const GValue *value,
                                        GParamSpec *pspec) {
  OverlaysModel *self = OVERLAYS_MODEL(object);
  switch (property_id) {
  case PROP_WEATHER:
    self->weather = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_WEATHER]);
    break;
  case PROP_SESSION:
    self->session = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_SESSION]);
    break;
  case PROP_TERMINAL:
    self->terminal = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_TERMINAL]);
    break;
  case PROP_PING:
    self->ping = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_PING]);
    break;
  case PROP_SOUND:
    self->sound = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_SOUND]);
    break;
  case PROP_CAPS_LOCK:
    self->caps_lock = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_CAPS_LOCK]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void overlays_model_init(OverlaysModel *self) {
  self->weather = false;
  self->session = false;
  self->terminal = false;
  self->ping = false;
  self->sound = false;
  self->caps_lock = false;
}

static void overlays_model_class_init(OverlaysModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = overlays_model_get_property;
  object_class->set_property = overlays_model_set_property;

  properties[PROP_WEATHER] = g_param_spec_boolean(
      "weather", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SESSION] = g_param_spec_boolean(
      "session", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_TERMINAL] = g_param_spec_boolean(
      "terminal", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_PING] = g_param_spec_boolean(
      "ping", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SOUND] = g_param_spec_boolean(
      "sound", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_CAPS_LOCK] = g_param_spec_boolean(
      "caps-lock", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

OverlaysModel *overlays_model_new(void) {
  return g_object_new(overlays_model_get_type(), NULL);
}
