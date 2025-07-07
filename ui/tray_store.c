#include "ui/tray_store.h"

struct tray_store_t {
  GHashTable *hashtable;
};

static void no_free(void *) {}

tray_store_t *tray_store_new() {
  tray_store_t *store = malloc(sizeof(tray_store_t));
  store->hashtable =
      g_hash_table_new_full(g_str_hash, g_str_equal, free, no_free);
  return store;
}

void tray_store_free(tray_store_t *tray_store) {
  g_hash_table_destroy(tray_store->hashtable);
  free(tray_store);
}

void tray_store_insert(tray_store_t *store, const char *key, GtkWidget *value) {
  g_hash_table_insert(store->hashtable, strdup(key), value);
}

GtkWidget *tray_store_lookup(tray_store_t *store, const char *key) {
  return g_hash_table_lookup(store->hashtable, key);
}

GtkWidget *tray_store_remove(tray_store_t *store, const char *key) {
  GtkWidget *to_be_removed = g_hash_table_lookup(store->hashtable, key);
  g_hash_table_remove(store->hashtable, key);
  return to_be_removed;
}
