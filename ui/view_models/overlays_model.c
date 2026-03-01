#include "ui/view_models/overlays_model.h"

struct _OverlaysModel {
  GObject parent_instance;

  gboolean weather_visible;
  gboolean session_visible;
  gboolean terminal_visible;
  gboolean ping_visible;
  gboolean sound_visible;
  gboolean caps_lock_visible;
};

G_DEFINE_TYPE(OverlaysModel, overlays_model, G_TYPE_OBJECT)

enum {
  PROP_WEATHER_VISIBLE = 1,
  PROP_SESSION_VISIBLE,
  PROP_TERMINAL_VISIBLE,
  PROP_PING_VISIBLE,
  PROP_SOUND_VISIBLE,
  PROP_CAPS_LOCK_VISIBLE,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void overlays_model_get_property(GObject *object, guint property_id,
                                        GValue *value, GParamSpec *pspec) {
  OverlaysModel *self = OVERLAYS_MODEL(object);
  switch (property_id) {
  case PROP_WEATHER_VISIBLE:
    g_value_set_boolean(value, self->weather_visible);
    break;
  case PROP_SESSION_VISIBLE:
    g_value_set_boolean(value, self->session_visible);
    break;
  case PROP_TERMINAL_VISIBLE:
    g_value_set_boolean(value, self->terminal_visible);
    break;
  case PROP_PING_VISIBLE:
    g_value_set_boolean(value, self->ping_visible);
    break;
  case PROP_SOUND_VISIBLE:
    g_value_set_boolean(value, self->sound_visible);
    break;
  case PROP_CAPS_LOCK_VISIBLE:
    g_value_set_boolean(value, self->caps_lock_visible);
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
  case PROP_WEATHER_VISIBLE:
    self->weather_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_WEATHER_VISIBLE]);
    break;
  case PROP_SESSION_VISIBLE:
    self->session_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_SESSION_VISIBLE]);
    break;
  case PROP_TERMINAL_VISIBLE:
    self->terminal_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_TERMINAL_VISIBLE]);
    break;
  case PROP_PING_VISIBLE:
    self->ping_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_PING_VISIBLE]);
    break;
  case PROP_SOUND_VISIBLE:
    self->sound_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_SOUND_VISIBLE]);
    break;
  case PROP_CAPS_LOCK_VISIBLE:
    self->caps_lock_visible = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_CAPS_LOCK_VISIBLE]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void overlays_model_init(OverlaysModel *self) {
  self->weather_visible = false;
  self->session_visible = false;
  self->terminal_visible = false;
  self->ping_visible = false;
  self->sound_visible = false;
  self->caps_lock_visible = false;
}

static void overlays_model_class_init(OverlaysModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = overlays_model_get_property;
  object_class->set_property = overlays_model_set_property;

  properties[PROP_WEATHER_VISIBLE] = g_param_spec_boolean(
      "weather_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SESSION_VISIBLE] = g_param_spec_boolean(
      "session_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_TERMINAL_VISIBLE] = g_param_spec_boolean(
      "terminal_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_PING_VISIBLE] = g_param_spec_boolean(
      "ping_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_SOUND_VISIBLE] = g_param_spec_boolean(
      "sound_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_CAPS_LOCK_VISIBLE] = g_param_spec_boolean(
      "caps_lock_visible", NULL, NULL, false,
      G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

OverlaysModel *overlays_model_new(void) {
  return g_object_new(overlays_model_get_type(), NULL);
}
