#include "bindings.h"
#include "css.h"
#include "gio/gio.h"
#include "glib-object.h"
#include "icons.h"
#include "widgets.h"
#include <glib-unix.h>
#include <gtk/gtk.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

void on_sigusr1(void) { layer_shell_io_on_sigusr1(); }

GtkApplication *app;

void on_app_activate(GApplication *app) {
  init_icons();
  init_widgets();

  activate_top_bar(app);
  activate_session_screen(app);
  activate_launcher(app);
  activate_networks(app);
  activate_htop(app);
  activate_weather(app);

  layer_shell_io_spawn_thread();
}

int main() {
  layer_shell_io_init_logger();
  layer_shell_io_init();

  g_unix_signal_add(SIGUSR1, G_SOURCE_FUNC(on_sigusr1), NULL);

  app = gtk_application_new("com.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);

  g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);
  g_signal_connect(app, "startup", load_css, NULL);

  g_application_run(G_APPLICATION(app), 0, NULL);

  return 0;
}
