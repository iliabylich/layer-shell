#include "bindings.h"
#include "css.h"
#include "htop-window.h"
#include "icons.h"
#include "launcher-window.h"
#include "network-window.h"
#include "session-window.h"
#include "top-bar-window.h"
#include "weather-window.h"
#include <gio/gio.h>
#include <glib-object.h>
#include <glib-unix.h>
#include <gtk/gtk.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

void on_sigusr1(void) { layer_shell_io_on_sigusr1(); }

GtkApplication *app;

int poll_events(void) {
  layer_shell_io_poll_events();
  return 1;
}

void on_app_activate(GApplication *app) {
  init_icons();

  init_top_bar_window();
  init_session_window();
  init_launcher_window();
  init_network_window();
  init_htop_window();
  init_weather_window();

  g_timeout_add(50, G_SOURCE_FUNC(poll_events), NULL);

  printf("Finished building widgets...\n");

  activate_top_bar_window(app);
  activate_session_window(app);
  activate_launcher_window(app);
  activate_network_window(app);
  activate_htop_window(app);
  activate_weather_window(app);

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
