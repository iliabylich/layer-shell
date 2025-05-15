#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_app.h"

struct _Tray {
  GtkBox parent_instance;
};

#define MAX_ICONS_COUNT 10

G_DEFINE_TYPE(Tray, tray, GTK_TYPE_BOX)

enum {
  TRIGGERED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void tray_class_init(TrayClass *klass) {
  signals[TRIGGERED] =
      g_signal_new("triggered", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
                   NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
}

static void tray_init(Tray *) {}

GtkWidget *tray_new() {
  // clang-format off
  return g_object_new(
      TRAY_TYPE,
      "orientation", GTK_ORIENTATION_HORIZONTAL,
      "spacing", 10,
      "css-classes", (const char *[]){"widget", "tray", "padded", NULL},
      "name", "Tray",
      NULL);
  // clang-format on
}

static void tray_cleanup(Tray *self) {
  GtkWidget *child = gtk_widget_get_first_child(GTK_WIDGET(self));
  while (child) {
    GtkWidget *grandchild = gtk_widget_get_first_child(child);
    while (grandchild) {
      GtkWidget *next = gtk_widget_get_next_sibling(grandchild);
      gtk_widget_unparent(grandchild);
      grandchild = next;
    }
    GtkWidget *next = gtk_widget_get_next_sibling(child);
    gtk_box_remove(GTK_BOX(self), child);
    child = next;
  }
}

static void tray_add(Tray *self, IO_TrayApp tray_app) {
  GtkWidget *icon = tray_app_new(tray_app, self);
  gtk_box_append(GTK_BOX(self), icon);
}

void tray_emit_triggered(Tray *tray, char *uuid) {
  g_signal_emit(tray, signals[TRIGGERED], 0, uuid);
}

void tray_refresh(Tray *self, IO_CArray_TrayApp apps) {
  tray_cleanup(self);

  for (size_t i = 0; i < apps.len && i < MAX_ICONS_COUNT; i++) {
    tray_add(self, apps.ptr[i]);
  }
}
