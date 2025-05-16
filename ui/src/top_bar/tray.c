#include "ui/include/top_bar/tray.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar/tray_app_icon.h"

struct _Tray {
  GtkBox parent_instance;

  GList *icons;
};

#define MAX_ICONS_COUNT 10

G_DEFINE_TYPE(Tray, tray, GTK_TYPE_BOX)

enum {
  TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void tray_cleanup(Tray *self);
static void tray_dispose(GObject *gobject) {
  Tray *self = TRAY(gobject);
  tray_cleanup(self);
  G_OBJECT_CLASS(tray_parent_class)->dispose(gobject);
}

static void tray_class_init(TrayClass *klass) {
  signals[TRIGGERED] =
      g_signal_new("triggered", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
                   NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);

  G_OBJECT_CLASS(klass)->dispose = tray_dispose;
}

static void tray_init(Tray *self) { self->icons = NULL; }

GtkWidget *tray_new() {
  // clang-format off
  return g_object_new(
      TRAY_TYPE,
      "orientation", GTK_ORIENTATION_HORIZONTAL,
      "spacing", 10,
      "css-classes", CSS("widget", "tray", "padded"),
      "name", "Tray",
      NULL);
  // clang-format on
}

static void tray_cleanup(Tray *self) {
  if (self->icons == NULL) {
    return;
  }

  for (GList *ptr = self->icons; ptr != NULL; ptr = ptr->next) {
    GtkWidget *icon = GTK_WIDGET(ptr->data);
    tray_app_icon_cleanup(TRAY_APP_ICON(icon));
  }
  g_list_free(self->icons);
  self->icons = NULL;
}

void tray_emit_triggered(Tray *tray, char *uuid) {
  g_signal_emit(tray, signals[TRIGGERED], 0, uuid);
}

void tray_refresh(Tray *self, IO_CArray_TrayApp apps) {
  tray_cleanup(self);

  self->icons = NULL;
  for (size_t i = 0; i < apps.len && i < MAX_ICONS_COUNT; i++) {
    GtkWidget *icon = tray_app_icon_new(apps.ptr[i], self);
    self->icons = g_list_append(self->icons, icon);
    gtk_box_append(GTK_BOX(self), icon);
  }
}
