#include "ui/cpu_item.h"

struct _CpuItem {
  GObject parent_instance;
  guint load;
};

G_DEFINE_TYPE(CpuItem, cpu_item, G_TYPE_OBJECT)

enum {
  PROP_LOAD = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void cpu_item_get_property(GObject *object, guint property_id,
                                  GValue *value, GParamSpec *pspec) {
  CpuItem *self = CPU_ITEM(object);
  switch (property_id) {
  case PROP_LOAD:
    g_value_set_uint(value, self->load);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void cpu_item_set_property(GObject *object, guint property_id,
                                  const GValue *value, GParamSpec *pspec) {
  CpuItem *self = CPU_ITEM(object);
  switch (property_id) {
  case PROP_LOAD:
    self->load = g_value_get_uint(value);
    g_object_notify_by_pspec(object, properties[PROP_LOAD]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void cpu_item_init(CpuItem *self) { self->load = 0; }

static void cpu_item_class_init(CpuItemClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = cpu_item_get_property;
  object_class->set_property = cpu_item_set_property;

  properties[PROP_LOAD] =
      g_param_spec_uint("load", NULL, NULL, 0, 100, 0, G_PARAM_READWRITE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

CpuItem *cpu_item_new(void) { return g_object_new(cpu_item_get_type(), NULL); }
