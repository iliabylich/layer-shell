#include "session.h"
#include "bindings.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) session_ns_##name

static GtkWindow *_(window);

static GtkWidget *_(lock_button);
static GtkWidget *_(reboot_button);
static GtkWidget *_(shutdown_button);
static GtkWidget *_(logout_button);

static GtkWidget *_(make_button)(const char *text) {
  GtkWidget *btn = gtk_button_new();
  gtk_widget_add_css_class(btn, "session-window-button");
  GtkWidget *label = gtk_label_new(text);
  gtk_button_set_child(GTK_BUTTON(btn), label);
  return btn;
}

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "SessionWindow");
  gtk_widget_add_css_class(GTK_WIDGET(_(window)), "session-window");

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_box_set_homogeneous(GTK_BOX(layout), true);
  gtk_box_set_spacing(GTK_BOX(layout), 200);
  gtk_widget_add_css_class(layout, "session-window-wrapper");
  gtk_window_set_child(_(window), layout);

  _(lock_button) = _(make_button)("Lock");
  gtk_box_append(GTK_BOX(layout), _(lock_button));

  _(reboot_button) = _(make_button)("Reboot");
  gtk_box_append(GTK_BOX(layout), _(reboot_button));

  _(shutdown_button) = _(make_button)("Shutdown");
  gtk_box_append(GTK_BOX(layout), _(shutdown_button));

  _(logout_button) = _(make_button)("Logout");
  gtk_box_append(GTK_BOX(layout), _(logout_button));
}

static void _(toggle)(void) { flip_window_visibility(_(window)); }

static void _(lock)(void) {
  _(toggle)();
  layer_shell_io_publish((IO_Command){.tag = IO_Command_Lock});
}
static void _(reboot)(void) {
  _(toggle)();
  layer_shell_io_publish((IO_Command){.tag = IO_Command_Reboot});
}
static void _(shutdown)(void) {
  _(toggle)();
  layer_shell_io_publish((IO_Command){.tag = IO_Command_Shutdown});
}
static void _(logout)(void) {
  _(toggle)();
  layer_shell_io_publish((IO_Command){.tag = IO_Command_Logout});
}

static void _(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                            GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    _(toggle)();
  }
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_ToggleSessionScreen: {
    _(toggle)();
    break;
  }
  default:
    break;
  }
}

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(_(window), "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(_(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(_(lock_button), "clicked", _(lock), NULL);
  g_signal_connect(_(reboot_button), "clicked", _(reboot), NULL);
  g_signal_connect(_(shutdown_button), "clicked", _(shutdown), NULL);
  g_signal_connect(_(logout_button), "clicked", _(logout), NULL);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(_(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(_(window)), ctrl);

  gtk_window_present(_(window));
  gtk_widget_set_visible(GTK_WIDGET(_(window)), false);

  layer_shell_io_subscribe(_(on_io_event));
}

window_t SESSION = {
    .init = _(init), .activate = _(activate), .toggle = _(toggle)};
