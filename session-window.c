#include "session-window.h"
#include "bindings.h"
#include "gtk4-layer-shell.h"
#include <gtk/gtk.h>

GtkWindow *session_window;
GtkButton *lock_button;
GtkButton *reboot_button;
GtkButton *shutdown_button;
GtkButton *logout_button;

static GtkButton *session_button_new(const char *text) {
  GtkButton *btn = GTK_BUTTON(gtk_button_new());
  gtk_widget_add_css_class(GTK_WIDGET(btn), "session-window-button");
  GtkLabel *label = GTK_LABEL(gtk_label_new(text));
  gtk_button_set_child(btn, GTK_WIDGET(label));
  return btn;
}

void init_session_window(void) {
  session_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(session_window), "SessionWindow");
  gtk_widget_add_css_class(GTK_WIDGET(session_window), "session-window");

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
  gtk_box_set_homogeneous(layout, true);
  gtk_box_set_spacing(layout, 200);
  gtk_widget_add_css_class(GTK_WIDGET(layout), "session-window-wrapper");
  gtk_window_set_child(session_window, GTK_WIDGET(layout));

  lock_button = session_button_new("Lock");
  gtk_box_append(layout, GTK_WIDGET(lock_button));

  reboot_button = session_button_new("Reboot");
  gtk_box_append(layout, GTK_WIDGET(reboot_button));

  shutdown_button = session_button_new("Shutdown");
  gtk_box_append(layout, GTK_WIDGET(shutdown_button));

  logout_button = session_button_new("Logout");
  gtk_box_append(layout, GTK_WIDGET(logout_button));
}

void toggle_session_window() {
  gtk_widget_set_visible(GTK_WIDGET(session_window),
                         !gtk_widget_get_visible(GTK_WIDGET(session_window)));
}

static void session_lock(void) {
  toggle_session_window();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Lock});
}
static void session_reboot(void) {
  toggle_session_window();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Reboot});
}
static void session_shutdown(void) {
  toggle_session_window();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Shutdown});
}
static void session_logout(void) {
  toggle_session_window();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Logout});
}

static void on_session_window_key_press(GtkEventControllerKey *self,
                                        guint keyval, guint keycode,
                                        GdkModifierType state,
                                        gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_session_window();
  }
}

static void session_window_on_event(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ToggleSessionScreen: {
    toggle_session_window();
    break;
  }
  default:
    break;
  }
}

void activate_session_window(GApplication *app) {
  gtk_window_set_application(session_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(session_window);
  gtk_layer_set_layer(session_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(session_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(session_window, "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(session_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(lock_button, "clicked", session_lock, NULL);
  g_signal_connect(reboot_button, "clicked", session_reboot, NULL);
  g_signal_connect(shutdown_button, "clicked", session_shutdown, NULL);
  g_signal_connect(logout_button, "clicked", session_logout, NULL);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(on_session_window_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(session_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  gtk_window_present(session_window);
  gtk_widget_set_visible(GTK_WIDGET(session_window), false);

  layer_shell_io_subscribe(session_window_on_event);
}
