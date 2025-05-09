#include "ui/include/window_helper.h"
#include <vte/vte.h>

void window_toggle(GtkWindow *window) {
  gtk_widget_set_visible(GTK_WIDGET(window),
                         !gtk_widget_get_visible(GTK_WIDGET(window)));
}

static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, GtkWindow *window) {
  const char *keyname = gdk_keyval_name(keyval);
  if (strcmp(keyname, "Escape") == 0) {
    window_toggle(window);
    return true;
  } else {
    return false;
  }
}

void window_toggle_on_escape(GtkWindow *window) {
  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(on_key_pressed), window);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(window), ctrl);
}

void vte_window(GtkWindow *window, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(window), "terminal-window");
  window_toggle_on_escape(window);

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(window, terminal);
}
