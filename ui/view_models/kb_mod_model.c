#include "ui/view_models/kb_mod_model.h"

struct _KbModModel {
  GObject parent_instance;
  IO_KbModKind kind;
  gboolean enabled;
  guint overlay_timer;
};

G_DEFINE_TYPE(KbModModel, kb_mod_model, G_TYPE_OBJECT)

enum {
  PROP_KIND = 1,
  PROP_ENABLED,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_OVERLAY_VISIBILITY_CHANGED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void hide_overlay(gpointer data) {
  KbModModel *self = KB_MOD_MODEL(data);
  self->overlay_timer = 0;
  g_signal_emit(self, signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED], 0, false);
}

static void request_overlay_show(KbModModel *self) {
  g_signal_emit(self, signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED], 0, true);
  if (self->overlay_timer != 0) {
    g_assert(g_source_remove(self->overlay_timer));
  }
  self->overlay_timer = g_timeout_add_once(1000, hide_overlay, self);
}

static void kb_mod_model_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  KbModModel *self = KB_MOD_MODEL(object);
  switch (property_id) {
  case PROP_KIND:
    g_value_set_uint(value, self->kind);
    break;
  case PROP_ENABLED:
    g_value_set_boolean(value, self->enabled);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void kb_mod_model_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  KbModModel *self = KB_MOD_MODEL(object);
  switch (property_id) {
  case PROP_KIND:
    self->kind = g_value_get_uint(value);
    g_object_notify_by_pspec(object, properties[PROP_KIND]);
    break;
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

static void kb_mod_model_init(KbModModel *self) {
  self->kind = IO_KbModKind_CapsLock;
  self->enabled = false;
  self->overlay_timer = 0;
}

static void kb_mod_model_finalize(GObject *object) {
  KbModModel *self = KB_MOD_MODEL(object);
  if (self->overlay_timer != 0) {
    g_assert(g_source_remove(self->overlay_timer));
    self->overlay_timer = 0;
  }
  G_OBJECT_CLASS(kb_mod_model_parent_class)->finalize(object);
}

static void kb_mod_model_class_init(KbModModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = kb_mod_model_get_property;
  object_class->set_property = kb_mod_model_set_property;
  object_class->finalize = kb_mod_model_finalize;

  properties[PROP_KIND] =
      g_param_spec_uint("kind", NULL, NULL, IO_KbModKind_CapsLock,
                        IO_KbModKind_NumLock, IO_KbModKind_CapsLock,
                        G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_ENABLED] = g_param_spec_boolean(
      "enabled", NULL, NULL, false, G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_OVERLAY_VISIBILITY_CHANGED] = g_signal_new(
      "overlay-visibility-changed", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST,
      0, NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_BOOLEAN);
}

KbModModel *kb_mod_model_new(void) {
  return g_object_new(kb_mod_model_get_type(), NULL);
}

void kb_mod_model_update(KbModModel *self, IO_KbModKind kind,
                         gboolean enabled) {
  g_object_set(self, "kind", kind, "enabled", enabled, NULL);
}
