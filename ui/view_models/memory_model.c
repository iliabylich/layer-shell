#include "ui/view_models/memory_model.h"

struct _MemoryModel {
  GObject parent_instance;

  double used;
  double total;
};

G_DEFINE_TYPE(MemoryModel, memory_model, G_TYPE_OBJECT)

enum {
  PROP_USED = 1,
  PROP_TOTAL,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void memory_model_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  MemoryModel *self = MEMORY_MODEL(object);
  switch (property_id) {
  case PROP_USED:
    g_value_set_double(value, self->used);
    break;
  case PROP_TOTAL:
    g_value_set_double(value, self->total);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void memory_model_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  MemoryModel *self = MEMORY_MODEL(object);
  switch (property_id) {
  case PROP_USED:
    self->used = g_value_get_double(value);
    g_object_notify_by_pspec(object, properties[PROP_USED]);
    break;
  case PROP_TOTAL:
    self->total = g_value_get_double(value);
    g_object_notify_by_pspec(object, properties[PROP_TOTAL]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void memory_model_init(MemoryModel *self) {
  self->used = 0.0;
  self->total = 0.0;
}

static void memory_model_class_init(MemoryModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = memory_model_get_property;
  object_class->set_property = memory_model_set_property;

  properties[PROP_USED] =
      g_param_spec_double("used", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  properties[PROP_TOTAL] =
      g_param_spec_double("total", NULL, NULL, 0.0, G_MAXDOUBLE, 0.0,
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

MemoryModel *memory_model_new(void) {
  return g_object_new(memory_model_get_type(), NULL);
}
