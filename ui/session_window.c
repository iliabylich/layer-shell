#include "ui/session_window.h"
#include "ui/logger.h"

LOGGER("SessionWindow", 0)

enum {
  SIGNAL_CLICKED_LOCK = 0,
  SIGNAL_CLICKED_REBOOT,
  SIGNAL_CLICKED_SHUTDOWN,
  SIGNAL_CLICKED_LOGOUT,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _SessionWindow {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(SessionWindow, session_window, BASE_WINDOW_TYPE)

static void lock_clicked(SessionWindow *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_LOCK], 0);
  base_window_set_window_visible(BASE_WINDOW(self), false);
}

static void reboot_clicked(SessionWindow *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_REBOOT], 0);
  base_window_set_window_visible(BASE_WINDOW(self), false);
}

static void shutdown_clicked(SessionWindow *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_SHUTDOWN], 0);
  base_window_set_window_visible(BASE_WINDOW(self), false);
}

static void logout_clicked(SessionWindow *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_LOGOUT], 0);
  base_window_set_window_visible(BASE_WINDOW(self), false);
}

static void session_window_init(SessionWindow *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void session_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(session_window_parent_class)->dispose(object);
}

static void session_window_class_init(SessionWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = session_window_dispose;

  signals[SIGNAL_CLICKED_LOCK] = g_signal_new_class_handler(
      "clicked-lock", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_REBOOT] = g_signal_new_class_handler(
      "clicked-reboot", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_SHUTDOWN] = g_signal_new_class_handler(
      "clicked-shutdown", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_LOGOUT] = g_signal_new_class_handler(
      "clicked-logout", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/session_window.ui");
  gtk_widget_class_bind_template_callback(widget_class, lock_clicked);
  gtk_widget_class_bind_template_callback(widget_class, reboot_clicked);
  gtk_widget_class_bind_template_callback(widget_class, shutdown_clicked);
  gtk_widget_class_bind_template_callback(widget_class, logout_clicked);
}

GtkWidget *session_window_new(GtkApplication *app, SessionWindowModel *state) {
  return g_object_new(session_window_get_type(), "application", app,
                      "window-state", state, NULL);
}
