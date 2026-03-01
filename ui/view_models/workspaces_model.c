#include "ui/view_models/workspace_item.h"
#include "ui/view_models/workspaces_model.h"

struct _WorkspacesModel {
  GObject parent_instance;

  GListStore *items;
  GtkFilterListModel *filtered;
};

G_DEFINE_TYPE(WorkspacesModel, workspaces_model, G_TYPE_OBJECT)

enum {
  PROP_VISIBLE = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void workspaces_model_get_property(GObject *object, guint property_id,
                                          GValue *value, GParamSpec *pspec) {
  WorkspacesModel *self = WORKSPACES_MODEL(object);
  switch (property_id) {
  case PROP_VISIBLE:
    g_value_set_object(value, self->filtered);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void workspaces_model_finalize(GObject *object) {
  WorkspacesModel *self = WORKSPACES_MODEL(object);
  g_clear_object(&self->items);
  g_clear_object(&self->filtered);
  G_OBJECT_CLASS(workspaces_model_parent_class)->finalize(object);
}

static gboolean ws_filter_func(gpointer item, gpointer) {
  gboolean visible;
  g_object_get(item, "visible", &visible, NULL);
  return visible;
}

static void workspaces_model_init(WorkspacesModel *self) {
  self->items = g_list_store_new(workspace_item_get_type());
  for (guint i = 1; i <= 10; i++) {
    WorkspaceItem *item = workspace_item_new(i);
    g_list_store_append(self->items, item);
    g_object_unref(item);
  }

  GtkCustomFilter *filter = gtk_custom_filter_new(ws_filter_func, NULL, NULL);
  self->filtered = gtk_filter_list_model_new(
      G_LIST_MODEL(g_object_ref(self->items)), GTK_FILTER(filter));
}

static void workspaces_model_class_init(WorkspacesModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = workspaces_model_get_property;
  object_class->finalize = workspaces_model_finalize;

  properties[PROP_VISIBLE] = g_param_spec_object(
      "visible", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

WorkspacesModel *workspaces_model_new(void) {
  return g_object_new(workspaces_model_get_type(), NULL);
}

void workspaces_model_update(WorkspacesModel *self,
                             struct IO_FFIArray_HyprlandWorkspace data) {
  guint n = g_list_model_get_n_items(G_LIST_MODEL(self->items));
  for (guint i = 0; i < n; i++) {
    WorkspaceItem *item = g_list_model_get_item(G_LIST_MODEL(self->items), i);
    g_object_set(item, "visible", (gboolean)data.ptr[i].visible, "active",
                 (gboolean)data.ptr[i].active, NULL);
    g_object_unref(item);
  }
  GtkFilter *filter = gtk_filter_list_model_get_filter(self->filtered);
  gtk_filter_changed(filter, GTK_FILTER_CHANGE_DIFFERENT);
}
