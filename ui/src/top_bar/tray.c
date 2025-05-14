#include "ui/include/top_bar/tray.h"
#include "gtk/gtk.h"
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

static const char *css_classes[] = {"widget", "tray", "padded", NULL};

static void tray_init(Tray *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 10);
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_name(GTK_WIDGET(self), "Tray");
}

GtkWidget *tray_new() { return g_object_new(tray_get_type(), NULL); }

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
