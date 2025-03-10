#include "include/windows/session.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

Session::Session(const Glib::RefPtr<Gtk::Application> &app, void *ctx)
    : utils::Subscriber(ctx), lock("Lock"), reboot("Reboot"),
      shutdown("Shutdown"), logout("Logout") {
  set_name("SessionWindow");
  set_css_classes({"session-window"});
  set_application(app);
  toggle_on_escape();

  Gtk::Box layout(Gtk::Orientation::HORIZONTAL, 200);
  layout.set_homogeneous(true);
  layout.set_css_classes({"wrapper"});
  set_child(layout);

  layout.append(lock);
  layout.append(reboot);
  layout.append(shutdown);
  layout.append(logout);

  auto window = gobj();
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(window, "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  lock.signal_clicked().connect([this, ctx]() {
    toggle();
    layer_shell_io::layer_shell_io_lock(ctx);
  });
  reboot.signal_clicked().connect([this, ctx]() {
    toggle();
    layer_shell_io::layer_shell_io_reboot(ctx);
  });
  shutdown.signal_clicked().connect([this, ctx]() {
    toggle();
    layer_shell_io::layer_shell_io_shutdown(ctx);
  });
  logout.signal_clicked().connect([this, ctx]() {
    toggle();
    layer_shell_io::layer_shell_io_logout(ctx);
  });
}

void Session::on_toggle_session_screen_event() { toggle(); }

} // namespace windows
