#include "htop-window.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

GtkWindow *htop_window;

const uint32_t HTOP_WINDOW_WIDTH = 1000;

static void htop_window_init(void) {
  htop_window = GTK_WINDOW(gtk_window_new());

  gtk_widget_set_name(GTK_WIDGET(htop_window), "HtopWindow");
  gtk_widget_add_css_class(GTK_WIDGET(htop_window), "widget-htop");

  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, HTOP_WINDOW_WIDTH);
  g_object_set_property(G_OBJECT(htop_window), "width-request", &width_request);

  GValue height_request = G_VALUE_INIT;
  g_value_init(&height_request, G_TYPE_INT);
  g_value_set_int(&height_request, 700);
  g_object_set_property(G_OBJECT(htop_window), "height-request",
                        &height_request);
}

static void htop_window_toggle(void) { flip_window_visibility(htop_window); }

static void
htop_window_on_key_press(__attribute__((unused)) GtkEventControllerKey *self,
                         guint keyval, __attribute__((unused)) guint keycode,
                         __attribute__((unused)) GdkModifierType state,
                         __attribute__((unused)) gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    htop_window_toggle();
  }
}

static void htop_window_activate(GApplication *app) {
  gtk_window_set_application(htop_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(htop_window);
  gtk_layer_set_layer(htop_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(htop_window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(htop_window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(htop_window, "LayerShell/Htop");
  gtk_layer_set_keyboard_mode(htop_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  VteTerminal *terminal = VTE_TERMINAL(vte_terminal_new());
  const char *home = getenv("HOME");
  char *argv[] = {"htop", NULL};
  vte_terminal_spawn_async(terminal, VTE_PTY_DEFAULT, home, argv, NULL,
                           G_SPAWN_DEFAULT, NULL, NULL, NULL, -1, NULL, NULL,
                           NULL);
  gtk_window_set_child(htop_window, GTK_WIDGET(terminal));

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(htop_window_on_key_press),
                   NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(htop_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  gtk_window_present(htop_window);
  gtk_widget_set_visible(GTK_WIDGET(htop_window), false);
}

static void htop_window_move(uint32_t margin_left, uint32_t margin_top) {
  gtk_layer_set_margin(htop_window, GTK_LAYER_SHELL_EDGE_LEFT, margin_left);
  gtk_layer_set_margin(htop_window, GTK_LAYER_SHELL_EDGE_TOP, margin_top);
}

uint32_t htop_window_width(void) { return HTOP_WINDOW_WIDTH; }

window_t HTOP = {.init = htop_window_init,
                 .activate = htop_window_activate,
                 .toggle = htop_window_toggle,
                 .move = htop_window_move,
                 .width = htop_window_width};
