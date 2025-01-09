#include "htop-window.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

#define ns(name) htop_ns_##name

static GtkWindow *ns(window);

static const uint32_t ns(WIDTH) = 1000;

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());

  gtk_widget_set_name(GTK_WIDGET(ns(window)), "HtopWindow");
  gtk_widget_add_css_class(GTK_WIDGET(ns(window)), "widget-htop");
  window_set_width_request(GTK_WINDOW(ns(window)), ns(WIDTH));
  window_set_height_request(GTK_WINDOW(ns(window)), 700);
}

static void ns(toggle)(void) { flip_window_visibility(ns(window)); }

static void ns(move)(uint32_t margin_left, uint32_t margin_top) {
  move_layer_window(ns(window), margin_left, margin_top);
}

static void ns(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                             GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    ns(toggle)();
  }
}

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(ns(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(ns(window), "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(ns(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkWidget *terminal = vte_terminal_new();
  const char *home = getenv("HOME");
  char *argv[] = {"htop", NULL};
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT, home, argv,
                           NULL, G_SPAWN_DEFAULT, NULL, NULL, NULL, -1, NULL,
                           NULL, NULL);
  gtk_window_set_child(ns(window), terminal);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(ns(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(ns(window)), ctrl);

  gtk_window_present(ns(window));
  gtk_widget_set_visible(GTK_WIDGET(ns(window)), false);
}

uint32_t ns(width)(void) { return ns(WIDTH); }

window_t HTOP = {.init = ns(init),
                 .activate = ns(activate),
                 .toggle = ns(toggle),
                 .move = ns(move),
                 .width = ns(width)};
