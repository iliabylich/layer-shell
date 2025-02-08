#include "include/windows/top-bar.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

TopBar::TopBar() : Gtk::Window() {
  set_name("TopBarWindow");
  set_css_classes({"window"});

  Gtk::CenterBox layout;

  layout.add_css_class("main-wrapper");
  set_child(layout);

  Gtk::Box left = Gtk::Box(Gtk::Orientation::HORIZONTAL, 8);
  Gtk::Box right = Gtk::Box(Gtk::Orientation::HORIZONTAL, 4);

  left.append(workspaces);
  right.append(tray);
  right.append(weather);
  right.append(htop);
  right.append(language);
  right.append(sound);
  right.append(cpu);
  right.append(memory);
  right.append(network);
  right.append(time);
  right.append(session);

  layout.set_start_widget(left);
  layout.set_end_widget(right);
}

void TopBar::activate(const Glib::RefPtr<Gtk::Application> &app,
                      void *subscriptions) {
  set_application(app);
  auto window = gobj();

  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(window, "LayerShell/TopBar");

  workspaces.activate(subscriptions);
  tray.activate(subscriptions);
  weather.activate(subscriptions);
  htop.activate(subscriptions);
  language.activate(subscriptions);
  sound.activate(subscriptions);
  cpu.activate(subscriptions);
  memory.activate(subscriptions);
  network.activate(subscriptions);
  time.activate(subscriptions);
  session.activate(subscriptions);

  present();
}

} // namespace windows
