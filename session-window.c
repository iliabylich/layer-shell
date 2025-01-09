#include "session-window.h"
#include "bindings.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define ns(name) session_ns_##name

GtkWindow *ns(window);

GtkWidget *ns(lock_button);
GtkWidget *ns(reboot_button);
GtkWidget *ns(shutdown_button);
GtkWidget *ns(logout_button);

static GtkWidget *ns(make_button)(const char *text) {
  GtkWidget *btn = gtk_button_new();
  gtk_widget_add_css_class(btn, "session-window-button");
  GtkWidget *label = gtk_label_new(text);
  gtk_button_set_child(GTK_BUTTON(btn), label);
  return btn;
}

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(ns(window)), "SessionWindow");
  gtk_widget_add_css_class(GTK_WIDGET(ns(window)), "session-window");

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_set_homogeneous(GTK_BOX(layout), true);
  gtk_box_set_spacing(GTK_BOX(layout), 200);
  gtk_widget_add_css_class(layout, "session-window-wrapper");
  gtk_window_set_child(ns(window), layout);

  ns(lock_button) = ns(make_button)("Lock");
  gtk_box_append(GTK_BOX(layout), ns(lock_button));

  ns(reboot_button) = ns(make_button)("Reboot");
  gtk_box_append(GTK_BOX(layout), ns(reboot_button));

  ns(shutdown_button) = ns(make_button)("Shutdown");
  gtk_box_append(GTK_BOX(layout), ns(shutdown_button));

  ns(logout_button) = ns(make_button)("Logout");
  gtk_box_append(GTK_BOX(layout), ns(logout_button));
}

static void ns(toggle)(void) { flip_window_visibility(ns(window)); }

static void ns(lock)(void) {
  ns(toggle)();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Lock});
}
static void ns(reboot)(void) {
  ns(toggle)();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Reboot});
}
static void ns(shutdown)(void) {
  ns(toggle)();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Shutdown});
}
static void ns(logout)(void) {
  ns(toggle)();
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = Logout});
}

static void ns(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                             GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    ns(toggle)();
  }
}

static void ns(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ToggleSessionScreen: {
    ns(toggle)();
    break;
  }
  default:
    break;
  }
}

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(ns(window), "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(ns(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(ns(lock_button), "clicked", ns(lock), NULL);
  g_signal_connect(ns(reboot_button), "clicked", ns(reboot), NULL);
  g_signal_connect(ns(shutdown_button), "clicked", ns(shutdown), NULL);
  g_signal_connect(ns(logout_button), "clicked", ns(logout), NULL);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(ns(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(ns(window)), ctrl);

  gtk_window_present(ns(window));
  gtk_widget_set_visible(GTK_WIDGET(ns(window)), false);

  layer_shell_io_subscribe(ns(on_io_event));
}

window_t SESSION = {
    .init = ns(init), .activate = ns(activate), .toggle = ns(toggle)};
