#include "ui/caps_lock_window.h"
#include "ui/logger.h"

LOGGER("CapsLockWindow", 0)

struct _CapsLockWindow {
  GtkWidget parent_instance;

  IOModel *model;

  guint timer;
};

G_DEFINE_TYPE(CapsLockWindow, caps_lock_window, BASE_WINDOW_TYPE)

enum {
  PROP_MODEL = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static char *format_caps_icon(GObject *, bool enabled) {
  return g_strdup(enabled ? "" : "");
}

static char *format_caps_label(GObject *, bool enabled) {
  return g_strdup(enabled ? "CapsLock ON" : "CapsLock OFF");
}

static void caps_lock_window_init(CapsLockWindow *self) {
  LOG("init");
  self->model = NULL;
  gtk_widget_init_template(GTK_WIDGET(self));
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

static void caps_lock_changed(GObject *, GParamSpec *, gpointer data) {
  show(CAPS_LOCK_WINDOW(data));
}

static void caps_lock_window_get_property(GObject *object, guint property_id,
                                          GValue *value, GParamSpec *pspec) {
  CapsLockWindow *self = CAPS_LOCK_WINDOW(object);
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, self->model);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void caps_lock_window_set_property(GObject *object, guint property_id,
                                          const GValue *value,
                                          GParamSpec *pspec) {
  CapsLockWindow *self = CAPS_LOCK_WINDOW(object);
  switch (property_id) {
  case PROP_MODEL:
    g_set_object(&self->model, g_value_get_object(value));
    g_signal_connect_object(self->model, "notify::caps-lock-enabled",
                            G_CALLBACK(caps_lock_changed), self, 0);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void caps_lock_window_dispose(GObject *object) {
  LOG("dispose");
  CapsLockWindow *self = CAPS_LOCK_WINDOW(object);
  g_clear_object(&self->model);
  G_OBJECT_CLASS(caps_lock_window_parent_class)->dispose(object);
}

static void caps_lock_window_class_init(CapsLockWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = caps_lock_window_get_property;
  object_class->set_property = caps_lock_window_set_property;
  object_class->dispose = caps_lock_window_dispose;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/caps_lock_window.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_caps_icon);
  gtk_widget_class_bind_template_callback(widget_class, format_caps_label);
}

GtkWidget *caps_lock_window_new(GtkApplication *app, IOModel *model) {
  return g_object_new(caps_lock_window_get_type(), "application", app, "model",
                      model, NULL);
}
