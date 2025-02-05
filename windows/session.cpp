#include "include/windows/session.hpp"
#include "bindings.hpp"
#include "gtk4-layer-shell.h"

namespace windows {

Gtk::Button make_button(const char *text) {
  Gtk::Button button;
  button.set_css_classes({"session-window-button"});
  Gtk::Label label(text);
  button.set_child(label);
  return button;
}

Session::Session() : Gtk::Window() {
  set_name("SessionWindow");
  set_css_classes({"session-window"});

  Gtk::Box layout(Gtk::Orientation::HORIZONTAL, 200);
  layout.set_homogeneous(true);
  layout.set_css_classes({"session-window-wrapper"});
  set_child(layout);

  lock = make_button("Lock");
  layout.append(lock);

  reboot = make_button("Reboot");
  layout.append(reboot);

  shutdown = make_button("Shutdown");
  layout.append(shutdown);

  logout = make_button("Logout");
  layout.append(logout);
}

void Session::activate(const Glib::RefPtr<Gtk::Application> &app) {
  set_application(app);

  auto window = gobj();
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_BOTTOM, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_namespace(window, "LayerShell/SessionScreen");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  lock.signal_clicked().connect([]() {
    toggle();
    layer_shell_io::layer_shell_io_lock();
  });
  reboot.signal_clicked().connect([]() {
    toggle();
    layer_shell_io::layer_shell_io_reboot();
  });
  shutdown.signal_clicked().connect([]() {
    toggle();
    layer_shell_io::layer_shell_io_shutdown();
  });
  logout.signal_clicked().connect([]() {
    toggle();
    layer_shell_io::layer_shell_io_logout();
  });

  toggle_on_escape();

  subscribe_to_io_events();
}

void Session::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::ToggleSessionScreen) {
    toggle();
  }
}

} // namespace windows
