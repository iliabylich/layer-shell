#include "ui/view_models/caps_lock_model.h"

struct _CapsLockModel {
  GObject parent_instance;
  gboolean enabled;
  guint overlay_timer;
};

G_DEFINE_TYPE(CapsLockModel, caps_lock_model, G_TYPE_OBJECT)

enum {
  PROP_ENABLED = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_OVERLAY_VISIBILITY_CHANGED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void hide_overlay(gpointer data) {
  CapsLockModel *self = CAPS_LOCK_MODEL(data);
  self->overlay_timer = 0;
  g_signal_emit(self, signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED], 0, false);
}

static void request_overlay_show(CapsLockModel *self) {
  g_signal_emit(self, signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED], 0, true);
  if (self->overlay_timer != 0) {
    g_assert(g_source_remove(self->overlay_timer));
  }
  self->overlay_timer = g_timeout_add_once(1000, hide_overlay, self);
}

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
    request_overlay_show(self);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void caps_lock_model_init(CapsLockModel *self) {
  self->enabled = false;
  self->overlay_timer = 0;
}

static void caps_lock_model_finalize(GObject *object) {
  CapsLockModel *self = CAPS_LOCK_MODEL(object);
  if (self->overlay_timer != 0) {
    g_assert(g_source_remove(self->overlay_timer));
    self->overlay_timer = 0;
  }
  G_OBJECT_CLASS(caps_lock_model_parent_class)->finalize(object);
}

static void caps_lock_model_class_init(CapsLockModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = caps_lock_model_get_property;
  object_class->set_property = caps_lock_model_set_property;
  object_class->finalize = caps_lock_model_finalize;

  properties[PROP_ENABLED] = g_param_spec_boolean(
      "enabled", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED] = g_signal_new(
      "overlay-visibility-changed", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST,
      0, NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_BOOLEAN);
}

CapsLockModel *caps_lock_model_new(void) {
  return g_object_new(caps_lock_model_get_type(), NULL);
}
