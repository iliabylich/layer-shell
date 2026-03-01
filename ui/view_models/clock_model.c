#include "ui/view_models/clock_model.h"

struct _ClockModel {
  GObject parent_instance;

  gint64 unix_seconds;
};

G_DEFINE_TYPE(ClockModel, clock_model, G_TYPE_OBJECT)

enum {
  PROP_UNIX_SECONDS = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void clock_model_get_property(GObject *object, guint property_id,
                                     GValue *value, GParamSpec *pspec) {
  ClockModel *self = CLOCK_MODEL(object);
  switch (property_id) {
  case PROP_UNIX_SECONDS:
    g_value_set_int64(value, self->unix_seconds);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void clock_model_set_property(GObject *object, guint property_id,
                                     const GValue *value, GParamSpec *pspec) {
  ClockModel *self = CLOCK_MODEL(object);
  switch (property_id) {
  case PROP_UNIX_SECONDS:
    self->unix_seconds = g_value_get_int64(value);
    g_object_notify_by_pspec(object, properties[PROP_UNIX_SECONDS]);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void clock_model_init(ClockModel *self) { self->unix_seconds = 0; }

static void clock_model_class_init(ClockModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = clock_model_get_property;
  object_class->set_property = clock_model_set_property;

  properties[PROP_UNIX_SECONDS] =
      g_param_spec_int64("unix_seconds", NULL, NULL, 0, G_MAXINT64, 0,
                         G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

ClockModel *clock_model_new(void) {
  return g_object_new(clock_model_get_type(), NULL);
}
