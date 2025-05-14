#include "ui/include/session.h"
#include "gtk/gtk.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

struct _Session {
  GtkWindow parent_instance;

  GtkWidget *lock;
  GtkWidget *reboot;
  GtkWidget *shutdown;
  GtkWidget *logout;
};

G_DEFINE_TYPE(Session, session, GTK_TYPE_WINDOW)

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

static void session_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(window, "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
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
  window_toggle_on_escape(GTK_WINDOW(self));
  session_init_layer(GTK_WINDOW(self));

  GtkWidget *layout =
      g_object_new(GTK_TYPE_BOX,
                   //
                   "orientation", GTK_ORIENTATION_HORIZONTAL,
                   //
                   "spacing", 200,
                   //
                   "homogeneous", true,
                   //
                   "css-classes", (const char *[]){"wrapper", NULL},
                   //
                   NULL);
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

Session *session_new(GtkApplication *app) {
  return g_object_new(session_get_type(),
                      //
                      "application", app,
                      //
                      "name", "SessionWindow",
                      //
                      "css-classes", (const char *[]){"session-window", NULL},
                      //
                      NULL);
}
