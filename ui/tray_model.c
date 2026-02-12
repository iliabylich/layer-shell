#include "ui/tray_model.h"
#include "ui/tray_app_item.h"

struct _TrayModel {
  GObject parent_instance;

  GListStore *apps;
};

G_DEFINE_TYPE(TrayModel, tray_model, G_TYPE_OBJECT)

enum {
  PROP_APPS = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void tray_model_get_property(GObject *object, guint property_id,
                                    GValue *value, GParamSpec *pspec) {
  TrayModel *self = TRAY_MODEL(object);
  switch (property_id) {
  case PROP_APPS:
    g_value_set_object(value, self->apps);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void tray_model_finalize(GObject *object) {
  TrayModel *self = TRAY_MODEL(object);
  g_clear_object(&self->apps);
  G_OBJECT_CLASS(tray_model_parent_class)->finalize(object);
}

static void tray_model_init(TrayModel *self) {
  self->apps = g_list_store_new(tray_app_item_get_type());
}

static void tray_model_class_init(TrayModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = tray_model_get_property;
  object_class->finalize = tray_model_finalize;

  properties[PROP_APPS] = g_param_spec_object(
      "apps", NULL, NULL, G_TYPE_LIST_MODEL, G_PARAM_READABLE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

TrayModel *tray_model_new(void) {
  return g_object_new(tray_model_get_type(), NULL);
}

static TrayAppItem *find_app(TrayModel *self, const char *service) {
  guint n = g_list_model_get_n_items(G_LIST_MODEL(self->apps));
  for (guint i = 0; i < n; i++) {
    TrayAppItem *item = g_list_model_get_item(G_LIST_MODEL(self->apps), i);
    if (strcmp(tray_app_item_get_service(item), service) == 0) {
      g_object_unref(item);
      return item;
    }
    g_object_unref(item);
  }
  return NULL;
}

static guint find_app_index(TrayModel *self, const char *service) {
  guint n = g_list_model_get_n_items(G_LIST_MODEL(self->apps));
  for (guint i = 0; i < n; i++) {
    TrayAppItem *item = g_list_model_get_item(G_LIST_MODEL(self->apps), i);
    gboolean match = strcmp(tray_app_item_get_service(item), service) == 0;
    g_object_unref(item);
    if (match)
      return i;
  }
  return GTK_INVALID_LIST_POSITION;
}

void tray_model_add_app(TrayModel *self, const char *service, IO_TrayIcon icon,
                        IO_FFIArray_TrayItem items) {
  TrayAppItem *item = tray_app_item_new(service, icon, items);
  g_list_store_append(self->apps, item);
  g_object_unref(item);
}

void tray_model_remove_app(TrayModel *self, const char *service) {
  guint idx = find_app_index(self, service);
  if (idx != GTK_INVALID_LIST_POSITION)
    g_list_store_remove(self->apps, idx);
}

void tray_model_update_icon(TrayModel *self, const char *service,
                            IO_TrayIcon icon) {
  TrayAppItem *item = find_app(self, service);
  if (item)
    tray_app_item_update_icon(item, icon);
}

void tray_model_update_menu(TrayModel *self, const char *service,
                            IO_FFIArray_TrayItem items) {
  TrayAppItem *item = find_app(self, service);
  if (item)
    tray_app_item_update_menu(item, items);
}
