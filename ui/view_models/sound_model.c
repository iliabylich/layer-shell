#include "ui/view_models/sound_model.h"

struct _SoundModel {
  GObject parent_instance;
  guint volume;
  gboolean muted;
  gboolean has_initial_state;
};

G_DEFINE_TYPE(SoundModel, sound_model, G_TYPE_OBJECT)

enum {
  PROP_VOLUME = 1,
  PROP_MUTED,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_OVERLAY_SHOW_REQUESTED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void request_overlay_show_if_initialized(SoundModel *self) {
  if (!self->has_initial_state) {
    return;
  }
  g_signal_emit(self, signals[SIGNAL_OVERLAY_SHOW_REQUESTED], 0);
}

static void sound_model_get_property(GObject *object, guint property_id,
                                     GValue *value, GParamSpec *pspec) {
  SoundModel *self = SOUND_MODEL(object);
  switch (property_id) {
  case PROP_VOLUME:
    g_value_set_uint(value, self->volume);
    break;
  case PROP_MUTED:
    g_value_set_boolean(value, self->muted);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_model_set_property(GObject *object, guint property_id,
                                     const GValue *value, GParamSpec *pspec) {
  SoundModel *self = SOUND_MODEL(object);
  switch (property_id) {
  case PROP_VOLUME:
    self->volume = g_value_get_uint(value);
    g_object_notify_by_pspec(object, properties[PROP_VOLUME]);
    request_overlay_show_if_initialized(self);
    break;
  case PROP_MUTED:
    self->muted = g_value_get_boolean(value);
    g_object_notify_by_pspec(object, properties[PROP_MUTED]);
    request_overlay_show_if_initialized(self);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void sound_model_init(SoundModel *self) {
  self->volume = 0;
  self->muted = false;
  self->has_initial_state = false;
}

static void sound_model_class_init(SoundModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = sound_model_get_property;
  object_class->set_property = sound_model_set_property;

  properties[PROP_VOLUME] =
      g_param_spec_uint("volume", NULL, NULL, 0, G_MAXUINT, 0,
                        G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_MUTED] = g_param_spec_boolean(
      "muted", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_OVERLAY_SHOW_REQUESTED] = g_signal_new(
      "overlay-show-requested", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
      NULL, NULL, NULL, G_TYPE_NONE, 0);
}

SoundModel *sound_model_new(void) {
  return g_object_new(sound_model_get_type(), NULL);
}

void sound_model_set_initial(SoundModel *self, guint volume, gboolean muted) {
  self->volume = volume;
  self->muted = muted;
  self->has_initial_state = true;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_VOLUME]);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_MUTED]);
}
