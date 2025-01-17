#include "bindings.h"
#include "utils/css.h"
#include "utils/icons.h"
#include "windows/htop.h"
#include "windows/launcher.h"
#include "windows/network.h"
#include "windows/session.h"
#include "windows/top-bar.h"
#include "windows/weather.h"
#include <gio/gio.h>
#include <glib-object.h>
#include <glib-unix.h>
#include <gtk/gtk.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

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
  layer_shell_io_init();

  g_unix_signal_add(SIGUSR1, G_SOURCE_FUNC(layer_shell_io_on_sigusr1), NULL);

  app = gtk_application_new("com.me.LayerShell", G_APPLICATION_DEFAULT_FLAGS);

  g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);
  g_signal_connect(app, "startup", load_css, NULL);

  g_application_run(G_APPLICATION(app), 0, NULL);

  return 0;
}
