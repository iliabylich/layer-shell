#include "ui/caps_lock_window.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

LOGGER("CapsLockWindow", 0)

struct _CapsLockWindow {
  GtkWidget parent_instance;

  GtkWidget *root;
  GtkWidget *icon;
  GtkWidget *label;

  guint timer;

  bool enabled;
};

G_DEFINE_TYPE(CapsLockWindow, caps_lock_window, BASE_WINDOW_TYPE)

static void caps_lock_window_init(CapsLockWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/CapsLock");
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, 100);
  gtk_widget_add_css_class(GTK_WIDGET(self), "caps-lock-window");
  gtk_widget_add_css_class(GTK_WIDGET(self), "notification-window");

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 20);
  gtk_widget_add_css_class(self->root, "wrapper");

  self->icon = gtk_label_new("?");
  gtk_widget_add_css_class(self->icon, "icon");
  gtk_box_append(GTK_BOX(self->root), self->icon);

  self->label = gtk_label_new("?");
  gtk_widget_add_css_class(self->label, "status");
  gtk_box_append(GTK_BOX(self->root), self->label);

  gtk_window_set_child(GTK_WINDOW(self), self->root);
}

static void caps_lock_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(caps_lock_window_parent_class)->dispose(object);
}

static void caps_lock_window_class_init(CapsLockWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = caps_lock_window_dispose;
}

GtkWidget *caps_lock_window_new(GtkApplication *app) {
  return g_object_new(caps_lock_window_get_type(), "application", app, NULL);
}

static void redraw(CapsLockWindow *self) {
  const char *icon;
  const char *label;

  if (self->enabled) {
    icon = "";
    label = "CapsLock ON";
  } else {
    icon = "";
    label = "CapsLock OFF";
  }

  gtk_label_set_label(GTK_LABEL(self->icon), icon);
  gtk_label_set_label(GTK_LABEL(self->label), label);
}

static void hide(gpointer data) {
  CapsLockWindow *self = data;
  gtk_widget_set_visible(GTK_WIDGET(self), false);
  self->timer = 0;
}

static void show(CapsLockWindow *self) {
  gtk_widget_set_visible(GTK_WIDGET(self), true);

  if (self->timer) {
    g_assert(g_source_remove(self->timer));
  }
  self->timer = g_timeout_add_once(1000, hide, self);
}

void caps_lock_window_refresh(CapsLockWindow *self,
                              IO_ControlCapsLockToggledEvent event) {
  self->enabled = event.enabled;
  redraw(self);
  show(self);
}
