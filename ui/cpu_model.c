#include "ui/cpu_model.h"
#include "ui/cpu_item.h"

struct _CpuModel {
  GObject parent_instance;

  GListStore *cores;
};

G_DEFINE_TYPE(CpuModel, cpu_model, G_TYPE_OBJECT)

enum {
  PROP_CORES = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void cpu_model_get_property(GObject *object, guint property_id,
                                   GValue *value, GParamSpec *pspec) {
  CpuModel *self = CPU_MODEL(object);
  switch (property_id) {
  case PROP_CORES:
    g_value_set_object(value, self->cores);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void cpu_model_finalize(GObject *object) {
  CpuModel *self = CPU_MODEL(object);
  g_clear_object(&self->cores);
  G_OBJECT_CLASS(cpu_model_parent_class)->finalize(object);
}

static void cpu_model_init(CpuModel *self) {
  self->cores = g_list_store_new(cpu_item_get_type());
}

static void cpu_model_class_init(CpuModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = cpu_model_get_property;
  object_class->finalize = cpu_model_finalize;

  properties[PROP_CORES] = g_param_spec_object(
      "cores", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

CpuModel *cpu_model_new(void) {
  return g_object_new(cpu_model_get_type(), NULL);
}

void cpu_model_update(CpuModel *self, IO_FFIArray_u8 data) {
  guint n = g_list_model_get_n_items(G_LIST_MODEL(self->cores));
  if (n == 0) {
    for (size_t i = 0; i < data.len; i++) {
      CpuItem *item = cpu_item_new();
      g_list_store_append(self->cores, item);
      g_object_unref(item);
    }
    n = data.len;
  }
  for (guint i = 0; i < n; i++) {
    CpuItem *item = g_list_model_get_item(G_LIST_MODEL(self->cores), i);
    g_object_set(item, "load", (guint)data.ptr[i], NULL);
    g_object_unref(item);
  }
}
