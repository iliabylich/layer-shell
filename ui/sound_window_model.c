#include "ui/sound_window_model.h"

struct _SoundWindowModel {
  WindowModel parent_instance;
  guint volume;
  gboolean muted;
  gboolean ready;
};

G_DEFINE_TYPE(SoundWindowModel, sound_window_model, window_model_get_type())

enum {
  PROP_VOLUME = 1,
  PROP_MUTED,
  PROP_READY,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void sound_window_model_get_property(GObject *object, guint property_id,
                                            GValue *value, GParamSpec *pspec) {
  SoundWindowModel *self = SOUND_WINDOW_MODEL(object);
  switch (property_id) {
  case PROP_VOLUME:
    g_value_set_uint(value, self->volume);
    break;
  case PROP_MUTED:
    g_value_set_boolean(value, self->muted);
    break;
  case PROP_READY:
    g_value_set_boolean(value, self->ready);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_window_model_set_property(GObject *object, guint property_id,
                                            const GValue *value,
                                            GParamSpec *pspec) {
  SoundWindowModel *self = SOUND_WINDOW_MODEL(object);
  switch (property_id) {
  case PROP_VOLUME:
    self->volume = g_value_get_uint(value);
    if (self->ready) {
      g_object_notify_by_pspec(object, properties[PROP_VOLUME]);
    }
    break;
  case PROP_MUTED:
    self->muted = g_value_get_boolean(value);
    if (self->ready) {
      g_object_notify_by_pspec(object, properties[PROP_MUTED]);
    }
    break;
  case PROP_READY:
    self->ready = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_READY]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_window_model_init(SoundWindowModel *self) {
  self->volume = 0;
  self->muted = false;
  self->ready = false;
}

static void sound_window_model_class_init(SoundWindowModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = sound_window_model_get_property;
  object_class->set_property = sound_window_model_set_property;

  properties[PROP_VOLUME] =
      g_param_spec_uint("volume", NULL, NULL, 0, G_MAXUINT, 0,
                        G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_MUTED] = g_param_spec_boolean(
      "muted", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_READY] = g_param_spec_boolean(
      "ready", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

SoundWindowModel *sound_window_model_new(void) {
  return g_object_new(sound_window_model_get_type(), NULL);
}
