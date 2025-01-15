#include "htop.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

#define _(name) htop_ns_##name

static GtkWindow *_(window);

static const int _(WIDTH) = 1000;

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());

  gtk_widget_set_name(GTK_WIDGET(_(window)), "HtopWindow");
  gtk_widget_add_css_class(GTK_WIDGET(_(window)), "widget-htop");
  window_set_width_request(GTK_WINDOW(_(window)), _(WIDTH));
  window_set_height_request(GTK_WINDOW(_(window)), 700);
}

static void _(toggle)(void) { flip_window_visibility(_(window)); }
static void _(move)(int x, int y) { move_layer_window(_(window), x, y); }

static void _(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                            GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    _(toggle)();
  }
}

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(_(window), "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(_(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkWidget *terminal = vte_terminal_new();
  const char *home = getenv("HOME");
  char *argv[] = {"htop", NULL};
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT, home, argv,
                           NULL, G_SPAWN_DEFAULT, NULL, NULL, NULL, -1, NULL,
                           NULL, NULL);
  gtk_window_set_child(_(window), terminal);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(_(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(_(window)), ctrl);

  gtk_window_present(_(window));
  gtk_widget_set_visible(GTK_WIDGET(_(window)), false);
}

window_t HTOP = {.init = _(init),
                 .activate = _(activate),
                 .toggle = _(toggle),
                 .move = _(move),
                 .width = _(WIDTH)};
