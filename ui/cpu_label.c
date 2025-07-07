#include "ui/cpu_label.h"
#include "ui/logger.h"

LOGGER("CpuLabel", 2)

enum {
  PROP_LOAD = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

struct _CpuLabel {
  GtkWidget parent_instance;

  GtkWidget *root;
};

G_DEFINE_TYPE(CpuLabel, cpu_label, GTK_TYPE_WIDGET)

static const char *INDICATORS[] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};
static const size_t INDICATORS_COUNT =
    sizeof(INDICATORS) / sizeof(const char *);

static void set_load(CpuLabel *self, float load) {
  size_t indicator_idx = floor(load / 100.0 * INDICATORS_COUNT);

  if (indicator_idx == INDICATORS_COUNT) {
    indicator_idx -= 1;
  }

  const char *markup = INDICATORS[indicator_idx];
  gtk_label_set_label(GTK_LABEL(self->root), markup);
}

static void cpu_label_init(CpuLabel *self) {
  LOG("init");

  self->root = gtk_label_new("");
  gtk_label_set_use_markup(GTK_LABEL(self->root), true);

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
  set_load(self, 0.0);
}

static void cpu_label_dispose(GObject *object) {
  LOG("dispose");

  CpuLabel *self = CPU_LABEL(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(cpu_label_parent_class)->dispose(object);
}

static void cpu_label_set_property(GObject *object, guint property_id,
                                   const GValue *value, GParamSpec *pspec) {
  CpuLabel *self = CPU_LABEL(object);

  switch (property_id) {
  case PROP_LOAD:
    uint load = g_value_get_uint(value);
    set_load(self, load);
    break;

  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void cpu_label_class_init(CpuLabelClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = cpu_label_dispose;
  object_class->set_property = cpu_label_set_property;

  properties[PROP_LOAD] =
      g_param_spec_uint("load", NULL, NULL, 0, 100, 0, G_PARAM_WRITABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *cpu_label_new(void) {
  return g_object_new(cpu_label_get_type(), NULL);
}
