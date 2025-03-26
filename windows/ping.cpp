#include "include/windows/ping.hpp"
#include "include/utils/strings.hpp"
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

namespace windows {

Ping::Ping(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *) {
  set_name("PingWindow");
  set_css_classes({"terminal-window"});
  property_width_request().set_value(1000);
  property_height_request().set_value(700);
  set_application(app);
  toggle_on_escape();

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(win, "LayerShell/Ping");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  auto terminal_raw = vte_terminal_new();
  const char *home = getenv("HOME");
  using utils::strings::s;
  char *argv[] = {s("ping"), s("8.8.8.8"), NULL};
  vte_terminal_spawn_async(VTE_TERMINAL(terminal_raw), VTE_PTY_DEFAULT, home,
                           argv, NULL, G_SPAWN_DEFAULT, NULL, NULL, NULL, -1,
                           NULL, NULL, NULL);

  terminal = Glib::wrap(terminal_raw);
  set_child(*terminal);
}

} // namespace windows
