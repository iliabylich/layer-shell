#include "ui/session_window.h"
#include "gtk/gtk.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include <gtk4-layer-shell.h>

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

  GtkWidget *root;

  GtkWidget *lock;
  GtkWidget *reboot;
  GtkWidget *shutdown;
  GtkWidget *logout;
};

G_DEFINE_TYPE(SessionWindow, session_window, BASE_WINDOW_TYPE)

static void on_click(GtkWidget *btn, SessionWindow *self) {
  guint signal;

  if (btn == self->lock) {
    signal = SIGNAL_CLICKED_LOCK;
  } else if (btn == self->reboot) {
    signal = SIGNAL_CLICKED_REBOOT;
  } else if (btn == self->shutdown) {
    signal = SIGNAL_CLICKED_SHUTDOWN;
  } else if (btn == self->logout) {
    signal = SIGNAL_CLICKED_LOGOUT;
  } else {
    return;
  }

  g_signal_emit(self, signals[signal], 0);
  session_window_toggle(self);
}

static void add_button(SessionWindow *self, GtkWidget **btn, const char *text) {
  *btn = gtk_button_new_with_label(text);
  g_signal_connect(*btn, "clicked", G_CALLBACK(on_click), self);
  gtk_box_append(GTK_BOX(self->root), *btn);
}

static void session_window_init(SessionWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
  gtk_widget_add_css_class(GTK_WIDGET(self), "session-window");
  g_object_set(G_OBJECT(self), "width-request", 400, "height-request", 300,
               NULL);

  base_window_set_toggle_on_escape(BASE_WINDOW(self));

  self->root = gtk_box_new(GTK_ORIENTATION_VERTICAL, 20);
  gtk_box_set_homogeneous(GTK_BOX(self->root), true);
  gtk_widget_add_css_class(self->root, "wrapper");

  add_button(self, &self->lock, "Lock");
  add_button(self, &self->reboot, "Reboot");
  add_button(self, &self->shutdown, "Shutdown");
  add_button(self, &self->logout, "Logout");

  gtk_window_set_child(GTK_WINDOW(self), self->root);
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
}

GtkWidget *session_window_new(GtkApplication *app) {
  return g_object_new(session_window_get_type(), "application", app, NULL);
}

void session_window_toggle(SessionWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}
