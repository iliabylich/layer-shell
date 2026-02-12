#include "ui/caps_lock_window.h"
#include "ui/logger.h"

LOGGER("CapsLockWindow", 0)

struct _CapsLockWindow {
  GtkWidget parent_instance;

  GtkWidget *icon;
  GtkWidget *label;

  guint timer;

  bool enabled;
};

G_DEFINE_TYPE(CapsLockWindow, caps_lock_window, BASE_WINDOW_TYPE)

static void caps_lock_window_init(CapsLockWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void caps_lock_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(caps_lock_window_parent_class)->dispose(object);
}

static void caps_lock_window_class_init(CapsLockWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = caps_lock_window_dispose;

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/caps_lock_window.ui");
  gtk_widget_class_bind_template_child(widget_class, CapsLockWindow, icon);
  gtk_widget_class_bind_template_child(widget_class, CapsLockWindow, label);
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

void caps_lock_window_refresh(CapsLockWindow *self, bool enabled) {
  self->enabled = enabled;
  redraw(self);
  show(self);
}
