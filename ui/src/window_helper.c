#include "ui/include/window_helper.h"
#include <vte/vte.h>

void vte_window(GtkWindow *window, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(window), "terminal-window");

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(window, terminal);
}
