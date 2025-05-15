#include "ui/include/session.h"
#include <gtk4-layer-shell.h>

struct _Session {
  BaseWindow parent_instance;

  GtkWidget *lock;
  GtkWidget *reboot;
  GtkWidget *shutdown;
  GtkWidget *logout;
};

G_DEFINE_TYPE(Session, session, BASE_WINDOW_TYPE)

enum {
  LOCK = 0,
  REBOOT,
  SHUTDOWN,
  LOGOUT,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void session_class_init(SessionClass *klass) {
#define SIGNAL(name, signal)                                                   \
  signals[signal] =                                                            \
      g_signal_new(name, G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0, NULL, \
                   NULL, NULL, G_TYPE_NONE, 0);

  SIGNAL("lock", LOCK);
  SIGNAL("reboot", REBOOT);
  SIGNAL("shutdown", SHUTDOWN);
  SIGNAL("logout", LOGOUT);
#undef SIGNAL
}

#define HANDLER(name, signal)                                                  \
  static void name(GtkButton *, Session *session) {                            \
    window_toggle(GTK_WINDOW(session));                                        \
    g_signal_emit(session, signals[signal], 0);                                \
  }

HANDLER(lock, LOCK)
HANDLER(reboot, REBOOT)
HANDLER(shutdown, SHUTDOWN)
HANDLER(logout, LOGOUT)
#undef HANDLER

static void session_init(Session *self) {
  // clang-format off
  GtkWidget *layout = g_object_new(
      GTK_TYPE_BOX,
      "orientation", GTK_ORIENTATION_HORIZONTAL,
      "spacing", 200,
      "homogeneous", true,
      "css-classes", (const char *[]){"wrapper", NULL},
      NULL);
  // clang-format on
  gtk_window_set_child(GTK_WINDOW(self), layout);

#define BUTTON(name, label)                                                    \
  self->name = gtk_button_new_with_label(label);                               \
  g_signal_connect(self->name, "clicked", G_CALLBACK(name), self);             \
  gtk_box_append(GTK_BOX(layout), self->name);

  BUTTON(lock, "Lock");
  BUTTON(reboot, "Reboot");
  BUTTON(shutdown, "Shutdown");
  BUTTON(logout, "Logout");
#undef BUTTON
}

GtkWidget *session_new(GtkApplication *app) {
  // clang-format off
  return g_object_new(
      SESSION_TYPE,
      "application", app,
      "name", "SessionWindow",
      "css-classes", (const char *[]){"session-window", NULL},
      "toggle-on-escape", true,
      "layer", GTK_LAYER_SHELL_LAYER_OVERLAY,
      "layer-anchor-top", true,
      "layer-anchor-right", true,
      "layer-anchor-bottom", true,
      "layer-anchor-left", true,
      "layer-namespace", "LayerShell/SessionScreen",
      "layer-keyboard-mode", GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE,
      NULL);
  // clang-format on
}
