#include "ui/include/top_bar/tray.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar.h"
#include "ui/include/top_bar/tray_app_icon.h"
#include <stdint.h>

typedef struct {
  GList *icons;
  tray_triggered_f callback;
} data_t;
#define DATA_KEY "data"
#define MAX_ICONS_COUNT 10

GtkWidget *tray_init(tray_triggered_f callback) {
  GtkWidget *self = top_bar_get_widget_by_id("TRAY");

  data_t *data = malloc(sizeof(data_t));
  data->icons = NULL;
  data->callback = callback;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  return self;
}

void tray_emit_triggered(GtkWidget *self, char *uuid) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->callback((const uint8_t *)uuid);
}

static void cleanup(GtkWidget *);

void tray_refresh(GtkWidget *self, IO_CArray_TrayApp apps) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  cleanup(self);

  data->icons = NULL;
  for (size_t i = 0; i < apps.len && i < MAX_ICONS_COUNT; i++) {
    GtkWidget *icon = tray_app_icon_new(apps.ptr[i], self);
    data->icons = g_list_append(data->icons, icon);
    gtk_box_append(GTK_BOX(self), icon);
  }
}

static void cleanup(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  if (data->icons == NULL) {
    return;
  }

  for (GList *ptr = data->icons; ptr != NULL; ptr = ptr->next) {
    GtkWidget *icon = GTK_WIDGET(ptr->data);
    tray_app_icon_cleanup(icon);
  }
  g_list_free(data->icons);
  data->icons = NULL;
}
