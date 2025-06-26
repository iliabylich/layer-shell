#include "ui/include/session.h"
#include "ui/include/builder.h"
#include "ui/include/utils/has_callback.h"
#include "ui/include/window_helper.h"
#include <gtk4-layer-shell.h>

WIDGET_HAS_CALLBACK(on_click_callback, on_session_btn_clicked_f)

static void on_click(GtkWidget *button, GtkWidget *session);

GtkWidget *session_init(GtkApplication *app,
                        on_session_btn_clicked_f lock_clicked_callback,
                        on_session_btn_clicked_f reboot_clicked_callback,
                        on_session_btn_clicked_f shutdown_clicked_callback,
                        on_session_btn_clicked_f logout_clicked_callback) {
  GtkWidget *self = session_get_widget("SESSION");
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

  GtkWidget *lock = session_get_widget("LOCK");
  set_on_click_callback(lock, lock_clicked_callback);
  g_signal_connect(lock, "clicked", G_CALLBACK(on_click), self);

  GtkWidget *reboot = session_get_widget("REBOOT");
  set_on_click_callback(reboot, reboot_clicked_callback);
  g_signal_connect(reboot, "clicked", G_CALLBACK(on_click), self);

  GtkWidget *shutdown = session_get_widget("SHUTDOWN");
  set_on_click_callback(shutdown, shutdown_clicked_callback);
  g_signal_connect(shutdown, "clicked", G_CALLBACK(on_click), self);

  GtkWidget *logout = session_get_widget("LOGOUT");
  set_on_click_callback(logout, logout_clicked_callback);
  g_signal_connect(logout, "clicked", G_CALLBACK(on_click), self);

  return self;
}

void session_toggle(GtkWidget *self) { window_toggle(GTK_WINDOW(self)); }

static void on_click(GtkWidget *button, GtkWidget *session) {
  session_toggle(session);
  get_on_click_callback(button)();
}
