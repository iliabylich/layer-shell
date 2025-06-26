#include "ui/include/top_bar/tray.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/top_bar/tray_store.h"

#define ICONS_KEY "icons"
#define TRIGGERED_CALLBACK_KEY "icons"

static tray_triggered_f tray_get_triggered_callback(GtkWidget *self) {
  return (tray_triggered_f)(size_t)g_object_get_data(G_OBJECT(self), ICONS_KEY);
}
static void tray_set_triggered_callback(GtkWidget *self,
                                        tray_triggered_f callback) {
  g_object_set_data(G_OBJECT(self), TRIGGERED_CALLBACK_KEY,
                    (void *)(size_t)callback);
}

GtkWidget *tray_init(tray_triggered_f callback) {
  GtkWidget *self = top_bar_get_widget("TRAY");

  tray_store_init(self);
  tray_set_triggered_callback(self, callback);

  return self;
}

static void tray_remove_service(GtkWidget *self, const char *service) {
  GtkWidget *existing = tray_store_remove(self, service);

  if (existing) {
    tray_app_icon_cleanup(existing);
  }
}

void tray_update_app(GtkWidget *self, IO_TrayAppUpdatedEvent event) {
  tray_remove_service(self, event.service);

  GtkWidget *new = tray_app_icon_new(event.icon, event.root_item,
                                     tray_get_triggered_callback(self));

  tray_store_insert(self, event.service, new);
  gtk_box_append(GTK_BOX(self), new);
}

void tray_remove_app(GtkWidget *self, IO_TrayAppRemovedEvent event) {
  tray_remove_service(self, event.service);
}
