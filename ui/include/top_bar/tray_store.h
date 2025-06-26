#pragma once

#include <gtk/gtk.h>

void tray_store_init(GtkWidget *tray);
GtkWidget *tray_store_lookup(GtkWidget *tray, const char *service);
GtkWidget *tray_store_remove(GtkWidget *tray, const char *service);
void tray_store_insert(GtkWidget *tray, const char *service, GtkWidget *icon);
