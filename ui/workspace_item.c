#include "ui/workspace_item.h"

struct _WorkspaceItem {
  GObject parent_instance;

  guint num;
  gboolean visible;
  gboolean active;
};

G_DEFINE_TYPE(WorkspaceItem, workspace_item, G_TYPE_OBJECT)

enum {
  PROP_NUM = 1,
  PROP_VISIBLE,
  PROP_ACTIVE,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void workspace_item_get_property(GObject *object, guint property_id,
                                        GValue *value, GParamSpec *pspec) {
  WorkspaceItem *self = WORKSPACE_ITEM(object);

  switch (property_id) {
  case PROP_NUM:
    g_value_set_uint(value, self->num);
    break;
  case PROP_VISIBLE:
    g_value_set_boolean(value, self->visible);
    break;
  case PROP_ACTIVE:
    g_value_set_boolean(value, self->active);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void workspace_item_set_property(GObject *object, guint property_id,
                                        const GValue *value,
                                        GParamSpec *pspec) {
  WorkspaceItem *self = WORKSPACE_ITEM(object);

  switch (property_id) {
  case PROP_NUM:
    self->num = g_value_get_uint(value);
    break;
  case PROP_VISIBLE:
    self->visible = g_value_get_boolean(value);
    break;
  case PROP_ACTIVE:
    self->active = g_value_get_boolean(value);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void workspace_item_init(WorkspaceItem *self) {
  self->num = 0;
  self->visible = FALSE;
  self->active = FALSE;
}

static void workspace_item_class_init(WorkspaceItemClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = workspace_item_get_property;
  object_class->set_property = workspace_item_set_property;

  properties[PROP_NUM] =
      g_param_spec_uint("num", NULL, NULL, 0, 100, 0, G_PARAM_READWRITE);
  properties[PROP_VISIBLE] =
      g_param_spec_boolean("visible", NULL, NULL, FALSE, G_PARAM_READWRITE);
  properties[PROP_ACTIVE] =
      g_param_spec_boolean("active", NULL, NULL, FALSE, G_PARAM_READWRITE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WorkspaceItem *workspace_item_new(guint num) {
  return g_object_new(workspace_item_get_type(), "num", num, NULL);
}
