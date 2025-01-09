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

  TOP_BAR.init();
  SESSION.init();
  LAUNCHER.init();
  NETWORK.init();
  HTOP.init();
  WEATHER.init();

  g_timeout_add(50, G_SOURCE_FUNC(poll_events), NULL);

  printf("Finished building widgets...\n");

  TOP_BAR.activate(app);
  SESSION.activate(app);
  LAUNCHER.activate(app);
  NETWORK.activate(app);
  HTOP.activate(app);
  WEATHER.activate(app);

  layer_shell_io_spawn_thread();
}

int main(void) {
  layer_shell_io_init_logger();
  layer_shell_io_init();

  g_unix_signal_add(SIGUSR1, G_SOURCE_FUNC(on_sigusr1), NULL);

  app = gtk_application_new("com.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);

  g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);
  g_signal_connect(app, "startup", load_css, NULL);

  g_application_run(G_APPLICATION(app), 0, NULL);

  return 0;
}
