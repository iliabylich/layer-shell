#include "ui/include/session.h"
#include "gtk/gtk.h"
#include "session.xml.xxd"
#include "ui/include/macros.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

typedef struct {
  on_lock_clicked_f lock_clicked_callback;
  on_reboot_clicked_f reboot_clicked_callback;
  on_shutdown_clicked_f shutdown_clicked_callback;
  on_logout_clicked_f logout_clicked_callback;
} data_t;
#define DATA_KEY "data"

BLP_BUILDER(session)

static void on_lock(GtkButton *, GtkWidget *self);
static void on_reboot(GtkButton *, GtkWidget *self);
static void on_shutdown(GtkButton *, GtkWidget *self);
static void on_logout(GtkButton *, GtkWidget *self);

GtkWidget *session_init(GtkApplication *app,
                        on_lock_clicked_f lock_clicked_callback,
                        on_reboot_clicked_f reboot_clicked_callback,
                        on_shutdown_clicked_f shutdown_clicked_callback,
                        on_logout_clicked_f logout_clicked_callback) {
  GtkWidget *self = builder_get_object("SESSION");
  gtk_window_set_application(GTK_WINDOW(self), app);
  window_set_toggle_on_escape(GTK_WINDOW(self));
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  data_t *data = malloc(sizeof(data_t));
  data->lock_clicked_callback = lock_clicked_callback;
  data->reboot_clicked_callback = reboot_clicked_callback;
  data->shutdown_clicked_callback = shutdown_clicked_callback;
  data->logout_clicked_callback = logout_clicked_callback;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  GtkWidget *lock = builder_get_object("LOCK");
  g_signal_connect(lock, "clicked", G_CALLBACK(on_lock), self);

  GtkWidget *reboot = builder_get_object("REBOOT");
  g_signal_connect(reboot, "clicked", G_CALLBACK(on_reboot), self);

  GtkWidget *shutdown = builder_get_object("SHUTDOWN");
  g_signal_connect(shutdown, "clicked", G_CALLBACK(on_shutdown), self);

  GtkWidget *logout = builder_get_object("LOGOUT");
  g_signal_connect(logout, "clicked", G_CALLBACK(on_logout), self);

  return self;
}

void session_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }

static void on_lock(GtkButton *, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  session_toggle(self);
  data->lock_clicked_callback();
}
static void on_reboot(GtkButton *, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  session_toggle(self);
  data->reboot_clicked_callback();
}
static void on_shutdown(GtkButton *, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  session_toggle(self);
  data->shutdown_clicked_callback();
}
static void on_logout(GtkButton *, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  session_toggle(self);
  data->logout_clicked_callback();
}
