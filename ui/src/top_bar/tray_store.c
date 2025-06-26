#include "ui/include/top_bar/tray_store.h"
#include "ui/include/utils/strclone.h"

static void no_free(void *) {}

#define KEY "store"

void tray_store_init(GtkWidget *tray) {
  GHashTable *store =
      g_hash_table_new_full(g_str_hash, g_str_equal, free, no_free);
  g_object_set_data_full(G_OBJECT(tray), KEY, store,
                         (GDestroyNotify)g_hash_table_destroy);
}

static GHashTable *tray_store_get(GtkWidget *tray) {
  return g_object_get_data(G_OBJECT(tray), KEY);
}

GtkWidget *tray_store_lookup(GtkWidget *tray, const char *service) {
  GHashTable *store = tray_store_get(tray);
  return g_hash_table_lookup(store, service);
}

GtkWidget *tray_store_remove(GtkWidget *tray, const char *service) {
  GHashTable *store = tray_store_get(tray);
  GtkWidget *to_be_removed = g_hash_table_lookup(store, service);
  g_hash_table_remove(store, service);
  return to_be_removed;
}

void tray_store_insert(GtkWidget *tray, const char *service, GtkWidget *icon) {
  GHashTable *store = tray_store_get(tray);
  g_hash_table_insert(store, strclone(service), icon);
}
