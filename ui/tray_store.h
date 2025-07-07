#pragma once

#include <gtk/gtk.h>

typedef struct tray_store_t tray_store_t;

tray_store_t *tray_store_new();
void tray_store_free(tray_store_t *store);
void tray_store_insert(tray_store_t *store, const char *key, GtkWidget *value);
GtkWidget *tray_store_lookup(tray_store_t *store, const char *key);
GtkWidget *tray_store_remove(tray_store_t *store, const char *key);
