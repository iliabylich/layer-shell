#include "ui/tray.h"
#include "ui/logger.h"
#include "ui/tray_icon.h"
#include "ui/tray_store.h"

LOGGER("Tray", 1)

enum {
  SIGNAL_TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _Tray {
  GtkWidget parent_instance;

  GtkWidget *root;
  tray_store_t *store;
};

G_DEFINE_TYPE(Tray, tray, GTK_TYPE_WIDGET)

static void on_trigger(GtkWidget *, const char *uuid, Tray *self) {
  g_signal_emit(self, signals[SIGNAL_TRIGGERED], 0, uuid);
}

static void tray_init(Tray *self) {
  LOG("init");

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "tray");
  gtk_widget_add_css_class(self->root, "padded");
  gtk_widget_set_cursor_from_name(self->root, "pointer");

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));

  self->store = tray_store_new();
}

static void tray_dispose(GObject *object) {
  LOG("dispose");

  Tray *self = TRAY(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  g_clear_pointer(&self->store, tray_store_free);
  G_OBJECT_CLASS(tray_parent_class)->dispose(object);
}

static void tray_class_init(TrayClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tray_dispose;
  signals[SIGNAL_TRIGGERED] = g_signal_new_class_handler(
      "triggered", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *tray_new(void) { return g_object_new(tray_get_type(), NULL); }

void tray_add_app(Tray *self, IO_FFIString service, IO_FFIArray_TrayItem items,
                  struct IO_TrayIcon icon) {
  GtkWidget *tray_icon = tray_icon_new(icon, items);
  tray_store_insert(self->store, service, tray_icon);
  gtk_box_append(GTK_BOX(self->root), tray_icon);
  g_signal_connect(tray_icon, "triggered", G_CALLBACK(on_trigger), self);
}

void tray_remove_app(Tray *self, IO_FFIString service) {
  GtkWidget *tray_icon = tray_store_remove(self->store, service);
  if (tray_icon == NULL) {
    return;
  }
  gtk_widget_unparent(tray_icon);
}

void tray_update_icon(Tray *self, IO_FFIString service,
                      struct IO_TrayIcon icon) {
  GtkWidget *tray_icon = tray_store_lookup(self->store, service);
  if (tray_icon == NULL) {
    return;
  }
  tray_icon_update_icon(TRAY_ICON(tray_icon), icon);
}

void tray_update_menu(Tray *self, IO_FFIString service,
                      IO_FFIArray_TrayItem items) {
  GtkWidget *tray_icon = tray_store_lookup(self->store, service);
  if (tray_icon == NULL) {
    return;
  }
  tray_icon_update_menu(TRAY_ICON(tray_icon), items);
}
